use super::config::build_lasm_postgres_thread_local_config;
use super::drop_lasm_db_tx_handle;
use super::lasm_db_records_capture_enabled;
use super::persist_lasm_db_record_after_unlock;
use super::records::{
    allocate_lasm_db_ephemeral_record, allocate_lasm_db_runtime_record,
    append_lasm_db_record_in_memory_with_compaction_snapshot,
    persist_lasm_db_record_with_capacity_guard,
};
use super::{run_lasm_db_tx_commit, run_lasm_db_tx_rollback};
use crate::lasm_db_runtime_common::allocate_lasm_db_tx_handle;
use crate::lasm_db_runtime_postgres::{
    connect_lasm_postgres_tx_client, discard_lasm_postgres_tx_client,
    parse_lasm_postgres_query_template_and_params,
    parse_lasm_postgres_query_template_and_params_value, put_lasm_postgres_tx_client,
    return_lasm_postgres_tx_client_to_pool, run_lasm_postgres_exec_thread_local,
    run_lasm_postgres_exec_tx_commit_on_client, run_lasm_postgres_exec_tx_on_client,
    run_lasm_postgres_exec_tx_one_shot_on_client, run_lasm_postgres_exec_tx_rollback_on_client,
    run_lasm_postgres_query_one_thread_local, take_lasm_postgres_tx_client_if_present,
    LasmPostgresParam, LasmPostgresThreadLocalClient, LasmPostgresThreadLocalConfig,
};
use crate::lasm_db_runtime_records_log::{
    build_lasm_records_log_query_one_row_object, find_lasm_records_log_latest_match,
};
use crate::lasm_db_runtime_sqlite::{
    parse_lasm_sqlite_query_params, parse_lasm_sqlite_query_params_value, run_lasm_sqlite_exec,
    run_lasm_sqlite_exec_tx, run_lasm_sqlite_query_one, LasmSqliteQueryParams,
};
use crate::{LasmDbRecord, LasmDbRecordsAdapter, LasmDynamicResponseState};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Mutex;

pub(crate) enum LasmPreparedDbOperationParams {
    Postgres {
        template: String,
        params: Vec<LasmPostgresParam>,
    },
    Sqlite {
        params: LasmSqliteQueryParams,
    },
    None,
}

pub(crate) enum LasmDbExecOperationResult {
    Sqlite { affected_rows: u64 },
    RecordsLog { affected_rows: u64 },
}

pub(crate) enum LasmDbExecTxOperationResult {
    Sqlite {
        affected_rows: u64,
        tx_started: bool,
    },
    RecordsLog {
        affected_rows: u64,
    },
}

#[derive(Debug)]
pub(crate) struct LasmDbExecTxError {
    pub(crate) message: String,
    pub(crate) tx_started: bool,
}

pub(crate) struct LasmPostgresExecTxClientSuccess {
    pub(crate) affected_rows: u64,
    pub(crate) active: bool,
    pub(crate) retained_client: Option<LasmPostgresThreadLocalClient>,
}

pub(crate) struct LasmPostgresExecTxClientError {
    pub(crate) message: String,
}

pub(crate) enum LasmDbQueryOneOperationResult {
    Sqlite { row: serde_json::Value },
    RecordsLog { row: serde_json::Value },
}

#[derive(Debug)]
pub(crate) enum LasmDbQueryOneOperationError {
    PreparationMismatch,
    NotFound,
    Runtime(String),
}

pub(crate) enum LasmUnlockedPostgresOperationError {
    StateUnavailable,
    AdapterMismatch,
    PreparationMismatch,
    NotFound,
    Runtime(String),
}

pub(crate) struct LasmUnlockedPostgresExecSuccess {
    pub(crate) record: LasmDbRecord,
}

pub(crate) struct LasmLockedExecSuccess {
    pub(crate) record: LasmDbRecord,
}

pub(crate) enum LasmExecTxSource {
    AllocateFromDb(i64),
    ExistingTx(i64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LasmInternalDbOperationSequenceValidationError {
    InvalidMarker,
    MarkerValueTooSmall,
    IndexedMarkersRequireCount,
    ExceedsMaximum { maximum: usize },
}

#[derive(Debug, Default)]
pub(crate) struct LasmInternalDbSequenceState {
    tx_handles_by_source: BTreeMap<i64, i64>,
    tx_handles: BTreeSet<i64>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct LasmInternalDbSequenceFailure {
    pub(crate) code: &'static str,
    pub(crate) kind: &'static str,
    pub(crate) message: &'static str,
    pub(crate) status: u16,
}

#[derive(Default)]
pub(crate) struct LasmSequenceExecTxPreparation {
    pub(crate) tx_header: Option<String>,
    pub(crate) tx_db_header: Option<String>,
    pub(crate) retain_tx_for_sequence: bool,
    pub(crate) allocated_tx_source: Option<i64>,
}

impl LasmInternalDbSequenceFailure {
    fn exec_tx_validation(message: &'static str) -> Self {
        Self {
            code: "DB.EXEC_TX_INVALID",
            kind: "validation",
            message,
            status: 400,
        }
    }

    fn tx_internal(message: &'static str) -> Self {
        Self {
            code: "DB.TX_INTERNAL",
            kind: "internal",
            message,
            status: 500,
        }
    }
}

pub(crate) fn validate_lasm_internal_db_operation_sequence_count(
    raw_operation_count: Option<String>,
    has_indexed_headers: bool,
    operation_sequence_max: usize,
) -> Result<usize, LasmInternalDbOperationSequenceValidationError> {
    let operation_count = if let Some(raw_operation_count) = raw_operation_count {
        let trimmed = raw_operation_count.trim();
        let Some(parsed) = trimmed.parse::<usize>().ok() else {
            return Err(LasmInternalDbOperationSequenceValidationError::InvalidMarker);
        };
        if parsed < 2 {
            return Err(LasmInternalDbOperationSequenceValidationError::MarkerValueTooSmall);
        }
        parsed
    } else {
        0
    };

    if operation_count == 0 && has_indexed_headers {
        return Err(LasmInternalDbOperationSequenceValidationError::IndexedMarkersRequireCount);
    }
    if operation_count > operation_sequence_max {
        return Err(
            LasmInternalDbOperationSequenceValidationError::ExceedsMaximum {
                maximum: operation_sequence_max,
            },
        );
    }

    Ok(operation_count)
}

impl LasmInternalDbSequenceState {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn tracked_tx_for_source(&self, db_source: i64) -> Option<i64> {
        self.tx_handles_by_source.get(&db_source).copied()
    }

    pub(crate) fn track_tx_handle(&mut self, tx_handle: i64) {
        self.tx_handles.insert(tx_handle);
    }

    pub(crate) fn track_source_tx_handle(&mut self, db_source: i64, tx_handle: i64) {
        self.tx_handles_by_source.insert(db_source, tx_handle);
    }

    pub(crate) fn cleanup(
        &self,
        dynamic_state: &Mutex<LasmDynamicResponseState>,
        db_records_adapter: LasmDbRecordsAdapter,
        operation_succeeded: bool,
    ) {
        super::cleanup_lasm_internal_db_sequence_tx_handles(
            dynamic_state,
            db_records_adapter,
            self.tx_handles_by_source.values().copied(),
            operation_succeeded,
        );
        super::cleanup_lasm_internal_db_sequence_tx_handles(
            dynamic_state,
            db_records_adapter,
            self.tx_handles.iter().copied(),
            operation_succeeded,
        );
    }

    pub(crate) fn prepare_exec_tx_sequence_operation<
        FMaterializeHeader,
        FParsePositiveI64,
        FIsValidDbCapHandle,
    >(
        &mut self,
        raw_tx_db: Option<String>,
        raw_tx_handle: Option<String>,
        mut materialize_header: FMaterializeHeader,
        parse_positive_i64: FParsePositiveI64,
        is_valid_db_cap_handle: FIsValidDbCapHandle,
    ) -> Result<LasmSequenceExecTxPreparation, LasmInternalDbSequenceFailure>
    where
        FMaterializeHeader: FnMut(String) -> String,
        FParsePositiveI64: Fn(&str) -> Option<i64>,
        FIsValidDbCapHandle: Fn(i64) -> bool,
    {
        if raw_tx_db.is_some() && raw_tx_handle.is_some() {
            return Err(LasmInternalDbSequenceFailure::exec_tx_validation(
                "db.execTx must include either tx handle or db.tx(dbCap) source, not both",
            ));
        }

        let mut preparation = LasmSequenceExecTxPreparation::default();
        if let Some(raw_tx_db_value) = raw_tx_db {
            let tx_db_source_raw = materialize_header(raw_tx_db_value.clone());
            let Some(tx_db_source) = parse_positive_i64(tx_db_source_raw.trim()) else {
                return Err(LasmInternalDbSequenceFailure::exec_tx_validation(
                    "db.execTx requires transaction and query handles",
                ));
            };
            if !is_valid_db_cap_handle(tx_db_source) {
                return Err(LasmInternalDbSequenceFailure::exec_tx_validation(
                    "db.execTx requires db.tx(dbCap) with valid db capability handle",
                ));
            }
            preparation.retain_tx_for_sequence = true;
            if let Some(existing_tx_handle) = self.tracked_tx_for_source(tx_db_source) {
                preparation.tx_header = Some(existing_tx_handle.to_string());
                self.track_tx_handle(existing_tx_handle);
            } else {
                preparation.tx_db_header = Some(raw_tx_db_value);
                preparation.allocated_tx_source = Some(tx_db_source);
            }
            return Ok(preparation);
        }

        if let Some(raw_tx_handle_value) = raw_tx_handle {
            let tx_handle_raw = materialize_header(raw_tx_handle_value.clone());
            let Some(tx_handle) = parse_positive_i64(tx_handle_raw.trim()) else {
                return Err(LasmInternalDbSequenceFailure::exec_tx_validation(
                    "db.execTx requires valid tx handle",
                ));
            };
            preparation.retain_tx_for_sequence = true;
            preparation.tx_header = Some(raw_tx_handle_value);
            self.track_tx_handle(tx_handle);
        }

        Ok(preparation)
    }

    pub(crate) fn track_sequence_tx_runtime_result<FMaterializeHeader, FParsePositiveI64>(
        &mut self,
        raw_db_source: Option<String>,
        raw_tx_result: Option<String>,
        mut materialize_header: FMaterializeHeader,
        parse_positive_i64: FParsePositiveI64,
    ) -> Result<(), LasmInternalDbSequenceFailure>
    where
        FMaterializeHeader: FnMut(String) -> String,
        FParsePositiveI64: Fn(&str) -> Option<i64>,
    {
        let Some(raw_db_source) = raw_db_source else {
            return Err(LasmInternalDbSequenceFailure::tx_internal(
                "db.tx runtime did not preserve db handle marker",
            ));
        };
        let db_source_raw = materialize_header(raw_db_source);
        let Some(db_source) = parse_positive_i64(db_source_raw.trim()) else {
            return Err(LasmInternalDbSequenceFailure::tx_internal(
                "db.tx runtime failure",
            ));
        };
        let Some(tx_handle_raw) = raw_tx_result else {
            return Err(LasmInternalDbSequenceFailure::tx_internal(
                "db.tx runtime did not publish transaction handle marker",
            ));
        };
        let Some(tx_handle) = parse_positive_i64(tx_handle_raw.as_str()) else {
            return Err(LasmInternalDbSequenceFailure::tx_internal(
                "db.tx runtime failure",
            ));
        };
        self.track_source_tx_handle(db_source, tx_handle);
        Ok(())
    }

    pub(crate) fn track_sequence_allocated_exec_tx_runtime_result<FParsePositiveI64>(
        &mut self,
        tx_db_source: i64,
        raw_tx_result: Option<String>,
        parse_positive_i64: FParsePositiveI64,
    ) -> Result<(), LasmInternalDbSequenceFailure>
    where
        FParsePositiveI64: Fn(&str) -> Option<i64>,
    {
        let Some(tx_handle_raw) = raw_tx_result else {
            return Err(LasmInternalDbSequenceFailure::tx_internal(
                "db.execTx runtime did not publish transaction handle marker",
            ));
        };
        let Some(tx_handle) = parse_positive_i64(tx_handle_raw.as_str()) else {
            return Err(LasmInternalDbSequenceFailure::tx_internal(
                "db.tx runtime failure",
            ));
        };
        self.track_source_tx_handle(tx_db_source, tx_handle);
        Ok(())
    }
}

pub(crate) struct LasmResolvedExecTxStateBindings {
    pub(crate) db: i64,
    pub(crate) tx: i64,
    pub(crate) tx_active: bool,
    pub(crate) allocated_tx_handle: Option<i64>,
}

pub(crate) struct LasmUnlockedPostgresQueryOneSuccess {
    pub(crate) record: LasmDbRecord,
    pub(crate) row_object: serde_json::Value,
}

pub(crate) struct LasmLockedQueryOneSuccess {
    pub(crate) record: LasmDbRecord,
    pub(crate) row_object: serde_json::Value,
}

pub(crate) struct LasmUnifiedExecSuccess {
    pub(crate) record: LasmDbRecord,
}

pub(crate) enum LasmUnifiedExecOperationError {
    StateUnavailable,
    AdapterMismatch,
    PreparationMismatch,
    Runtime(String),
}

pub(crate) struct LasmUnifiedQueryOneSuccess {
    pub(crate) record: LasmDbRecord,
    pub(crate) row_object: serde_json::Value,
}

pub(crate) enum LasmUnifiedQueryOneOperationError {
    StateUnavailable,
    AdapterMismatch,
    PreparationMismatch,
    NotFound,
    Runtime(String),
}

pub(crate) struct LasmUnifiedExecTxSuccess {
    pub(crate) record: LasmDbRecord,
}

pub(crate) enum LasmUnifiedExecTxOperationError {
    StateUnavailable,
    AdapterMismatch,
    PreparationMismatch,
    Runtime(String),
}

pub(crate) struct LasmUnlockedPostgresExecTxSuccess {
    pub(crate) record: LasmDbRecord,
}

pub(crate) enum LasmLockedOperationError {
    StateUnavailable,
    AdapterMismatch,
    Capacity { max_handles: usize },
    InvalidHandle,
    ConflictInUse,
    Runtime(String),
}

pub(crate) enum LasmLockedQueryOneOperationError {
    StateUnavailable,
    AdapterMismatch,
    NotFound,
    PreparationMismatch,
    Runtime(String),
}

pub(crate) enum LasmUnlockedPostgresExecTxOperationError {
    StateUnavailable,
    AdapterMismatch,
    PreparationMismatch,
    Runtime(String),
}

pub(crate) struct LasmLockedExecTxSuccess {
    pub(crate) record: LasmDbRecord,
}

pub(crate) enum LasmLockedExecTxOperationError {
    StateUnavailable,
    AdapterMismatch,
    Runtime(String),
}

pub(crate) fn parse_lasm_db_template_and_params(
    adapter: LasmDbRecordsAdapter,
    template: &str,
    params: &str,
    parsed_params: Option<&serde_json::Value>,
) -> Result<LasmPreparedDbOperationParams, String> {
    match adapter {
        LasmDbRecordsAdapter::Postgres => {
            let result = if let Some(parsed) = parsed_params {
                parse_lasm_postgres_query_template_and_params_value(template, parsed)
            } else {
                parse_lasm_postgres_query_template_and_params(template, params)
            };
            let (template, query_params) = result?;
            Ok(LasmPreparedDbOperationParams::Postgres {
                template,
                params: query_params,
            })
        }
        LasmDbRecordsAdapter::Sqlite => {
            let query_params = if let Some(parsed) = parsed_params {
                parse_lasm_sqlite_query_params_value(parsed)
            } else {
                parse_lasm_sqlite_query_params(params)
            }?;
            Ok(LasmPreparedDbOperationParams::Sqlite {
                params: query_params,
            })
        }
        LasmDbRecordsAdapter::RecordsLog => Ok(LasmPreparedDbOperationParams::None),
    }
}

pub(crate) fn run_lasm_db_exec_operation(
    state: &mut LasmDynamicResponseState,
    db_records_adapter: LasmDbRecordsAdapter,
    template: &str,
    prepared_params: &LasmPreparedDbOperationParams,
) -> Result<LasmDbExecOperationResult, String> {
    match db_records_adapter {
        LasmDbRecordsAdapter::Postgres => Err(
            "internal postgres exec must run through the unlocked thread-local path".to_string(),
        ),
        LasmDbRecordsAdapter::Sqlite => {
            let LasmPreparedDbOperationParams::Sqlite { params } = prepared_params else {
                return Err("internal db operation preparation mismatch".to_string());
            };
            let affected_rows = run_lasm_sqlite_exec(state, template, params)?;
            Ok(LasmDbExecOperationResult::Sqlite { affected_rows })
        }
        LasmDbRecordsAdapter::RecordsLog => {
            Ok(LasmDbExecOperationResult::RecordsLog { affected_rows: 0 })
        }
    }
}

pub(crate) fn run_lasm_db_exec_tx_operation(
    state: &mut LasmDynamicResponseState,
    db_records_adapter: LasmDbRecordsAdapter,
    tx: i64,
    tx_active: bool,
    template: &str,
    prepared_params: &LasmPreparedDbOperationParams,
) -> Result<LasmDbExecTxOperationResult, LasmDbExecTxError> {
    let to_error = |message: String, tx_started: bool| {
        Err(LasmDbExecTxError {
            message,
            tx_started,
        })
    };
    match db_records_adapter {
        LasmDbRecordsAdapter::Postgres => Err(LasmDbExecTxError {
            message: "internal postgres execTx must run through the unlocked helper path"
                .to_string(),
            tx_started: false,
        }),
        LasmDbRecordsAdapter::Sqlite => {
            let LasmPreparedDbOperationParams::Sqlite { params } = prepared_params else {
                return to_error(
                    "internal db operation preparation mismatch".to_string(),
                    false,
                );
            };
            match run_lasm_sqlite_exec_tx(state, tx, template, params, tx_active) {
                Ok((affected_rows, tx_started)) => Ok(LasmDbExecTxOperationResult::Sqlite {
                    affected_rows,
                    tx_started,
                }),
                Err((message, tx_started)) => Err(LasmDbExecTxError {
                    message,
                    tx_started,
                }),
            }
        }
        LasmDbRecordsAdapter::RecordsLog => {
            Ok(LasmDbExecTxOperationResult::RecordsLog { affected_rows: 0 })
        }
    }
}

pub(crate) fn run_lasm_postgres_exec_tx_client_operation(
    config: &LasmPostgresThreadLocalConfig,
    tx: i64,
    tx_active: bool,
    keep_allocated_tx_handle: bool,
    template: &str,
    params: &[LasmPostgresParam],
    existing_tx_client: Option<LasmPostgresThreadLocalClient>,
) -> Result<LasmPostgresExecTxClientSuccess, LasmPostgresExecTxClientError> {
    let mut tx_client = Some(match existing_tx_client {
        Some(client) => client,
        None => connect_lasm_postgres_tx_client(config)
            .map_err(|message| LasmPostgresExecTxClientError { message })?,
    });
    if !tx_active && !keep_allocated_tx_handle {
        let affected_rows = match run_lasm_postgres_exec_tx_one_shot_on_client(
            tx_client
                .as_mut()
                .expect("postgres tx client should exist during one-shot execTx"),
            template,
            params,
            config.retryable_conflict_retry_max,
        ) {
            Ok(affected_rows) => affected_rows,
            Err(err) => {
                if err.discard_client {
                    if let Some(client) = tx_client.take() {
                        discard_lasm_postgres_tx_client(config, client);
                    }
                } else if let Some(client) = tx_client.take() {
                    return_lasm_postgres_tx_client_to_pool(config, client);
                }
                return Err(LasmPostgresExecTxClientError {
                    message: err.message,
                });
            }
        };
        return_lasm_postgres_tx_client_to_pool(
            config,
            tx_client
                .take()
                .expect("postgres tx client should exist when returning one-shot client"),
        );
        return Ok(LasmPostgresExecTxClientSuccess {
            affected_rows,
            active: false,
            retained_client: None,
        });
    }
    let operation_result = run_lasm_postgres_exec_tx_on_client(
        tx_client
            .as_mut()
            .expect("postgres tx client should exist during execTx"),
        tx,
        template,
        params,
        tx_active,
        config.retryable_conflict_retry_max,
    );
    let (affected_rows, tx_started) = match operation_result {
        Ok(value) => value,
        Err((message, tx_started)) => {
            let rollback_succeeded = if tx_started || tx_active {
                run_lasm_postgres_exec_tx_rollback_on_client(
                    tx_client
                        .as_mut()
                        .expect("postgres tx client should exist during rollback"),
                    tx,
                )
                .is_ok()
            } else {
                false
            };
            if rollback_succeeded {
                return_lasm_postgres_tx_client_to_pool(
                    config,
                    tx_client
                        .take()
                        .expect("postgres tx client should exist when returning to pool"),
                );
            } else if let Some(client) = tx_client.take() {
                discard_lasm_postgres_tx_client(config, client);
            }
            return Err(LasmPostgresExecTxClientError { message });
        }
    };

    let should_finalize_postgres_tx = !keep_allocated_tx_handle && (tx_started || tx_active);
    if should_finalize_postgres_tx {
        if let Err(message) = run_lasm_postgres_exec_tx_commit_on_client(
            tx_client
                .as_mut()
                .expect("postgres tx client should exist during commit"),
            tx,
        ) {
            if let Some(client) = tx_client.take() {
                discard_lasm_postgres_tx_client(config, client);
            }
            return Err(LasmPostgresExecTxClientError { message });
        }
        return_lasm_postgres_tx_client_to_pool(
            config,
            tx_client
                .take()
                .expect("postgres tx client should exist when returning committed client"),
        );
        Ok(LasmPostgresExecTxClientSuccess {
            affected_rows,
            active: false,
            retained_client: None,
        })
    } else {
        Ok(LasmPostgresExecTxClientSuccess {
            affected_rows,
            active: tx_started || tx_active,
            retained_client: tx_client.take(),
        })
    }
}

pub(crate) fn run_lasm_postgres_exec_unlocked_operation(
    dynamic_state: &Mutex<LasmDynamicResponseState>,
    db: i64,
    template: &str,
    params: &str,
    prepared_params: &LasmPreparedDbOperationParams,
) -> Result<LasmUnlockedPostgresExecSuccess, LasmUnlockedPostgresOperationError> {
    let LasmPreparedDbOperationParams::Postgres {
        template: postgres_template,
        params: postgres_params,
    } = prepared_params
    else {
        return Err(LasmUnlockedPostgresOperationError::PreparationMismatch);
    };
    let config = {
        let state = dynamic_state
            .lock()
            .map_err(|_| LasmUnlockedPostgresOperationError::StateUnavailable)?;
        if state.db_records_adapter != LasmDbRecordsAdapter::Postgres {
            return Err(LasmUnlockedPostgresOperationError::AdapterMismatch);
        }
        build_lasm_postgres_thread_local_config(&state)
            .map_err(LasmUnlockedPostgresOperationError::Runtime)?
    };
    let affected_rows = run_lasm_postgres_exec_thread_local(
        &config,
        postgres_template.as_str(),
        postgres_params.as_slice(),
    )
    .map_err(LasmUnlockedPostgresOperationError::Runtime)?;
    if !lasm_db_records_capture_enabled() {
        return Ok(LasmUnlockedPostgresExecSuccess {
            record: allocate_lasm_db_ephemeral_record(
                "exec",
                db,
                template,
                params,
                0,
                affected_rows,
            ),
        });
    }
    let (record, compaction_snapshot) = {
        let mut state = dynamic_state
            .lock()
            .map_err(|_| LasmUnlockedPostgresOperationError::StateUnavailable)?;
        if state.db_records_adapter != LasmDbRecordsAdapter::Postgres {
            return Err(LasmUnlockedPostgresOperationError::AdapterMismatch);
        }
        let record = allocate_lasm_db_runtime_record(
            &mut state,
            "exec",
            db,
            template,
            params,
            0,
            affected_rows,
        );
        append_lasm_db_record_in_memory_with_compaction_snapshot(&mut state, record)
    };
    persist_lasm_db_record_after_unlock(
        LasmDbRecordsAdapter::Postgres,
        &config,
        &record,
        compaction_snapshot,
    );
    Ok(LasmUnlockedPostgresExecSuccess { record })
}

pub(crate) fn run_lasm_non_postgres_exec_locked_operation(
    dynamic_state: &Mutex<LasmDynamicResponseState>,
    db_records_adapter: LasmDbRecordsAdapter,
    db: i64,
    template: &str,
    params: &str,
    prepared_params: &LasmPreparedDbOperationParams,
) -> Result<LasmLockedExecSuccess, LasmLockedOperationError> {
    let mut state = dynamic_state
        .lock()
        .map_err(|_| LasmLockedOperationError::StateUnavailable)?;
    if state.db_records_adapter != db_records_adapter
        || db_records_adapter == LasmDbRecordsAdapter::Postgres
    {
        return Err(LasmLockedOperationError::AdapterMismatch);
    }
    let operation_result =
        run_lasm_db_exec_operation(&mut state, db_records_adapter, template, prepared_params)
            .map_err(LasmLockedOperationError::Runtime)?;
    let affected_rows = match operation_result {
        LasmDbExecOperationResult::Sqlite { affected_rows }
        | LasmDbExecOperationResult::RecordsLog { affected_rows } => affected_rows,
    };
    let record =
        allocate_lasm_db_runtime_record(&mut state, "exec", db, template, params, 0, affected_rows);
    persist_lasm_db_record_with_capacity_guard(&mut state, &record);
    Ok(LasmLockedExecSuccess { record })
}

pub(crate) fn run_lasm_exec_operation_with_adapter(
    dynamic_state: &Mutex<LasmDynamicResponseState>,
    db_records_adapter: LasmDbRecordsAdapter,
    db: i64,
    template: &str,
    params: &str,
    prepared_params: &LasmPreparedDbOperationParams,
) -> Result<LasmUnifiedExecSuccess, LasmUnifiedExecOperationError> {
    if db_records_adapter == LasmDbRecordsAdapter::Postgres {
        return run_lasm_postgres_exec_unlocked_operation(
            dynamic_state,
            db,
            template,
            params,
            prepared_params,
        )
        .map(|success| LasmUnifiedExecSuccess {
            record: success.record,
        })
        .map_err(|error| match error {
            LasmUnlockedPostgresOperationError::StateUnavailable => {
                LasmUnifiedExecOperationError::StateUnavailable
            }
            LasmUnlockedPostgresOperationError::AdapterMismatch => {
                LasmUnifiedExecOperationError::AdapterMismatch
            }
            LasmUnlockedPostgresOperationError::PreparationMismatch => {
                LasmUnifiedExecOperationError::PreparationMismatch
            }
            LasmUnlockedPostgresOperationError::Runtime(message) => {
                LasmUnifiedExecOperationError::Runtime(message)
            }
            LasmUnlockedPostgresOperationError::NotFound => LasmUnifiedExecOperationError::Runtime(
                "postgres exec returned unexpected not-found result".to_string(),
            ),
        });
    }

    run_lasm_non_postgres_exec_locked_operation(
        dynamic_state,
        db_records_adapter,
        db,
        template,
        params,
        prepared_params,
    )
    .map(|success| LasmUnifiedExecSuccess {
        record: success.record,
    })
    .map_err(|error| match error {
        LasmLockedOperationError::StateUnavailable => {
            LasmUnifiedExecOperationError::StateUnavailable
        }
        LasmLockedOperationError::AdapterMismatch => LasmUnifiedExecOperationError::AdapterMismatch,
        LasmLockedOperationError::Capacity { .. }
        | LasmLockedOperationError::InvalidHandle
        | LasmLockedOperationError::ConflictInUse => LasmUnifiedExecOperationError::Runtime(
            "internal non-postgres exec control-state mismatch".to_string(),
        ),
        LasmLockedOperationError::Runtime(message) => {
            LasmUnifiedExecOperationError::Runtime(message)
        }
    })
}

pub(crate) fn run_lasm_db_tx_allocate_locked_operation(
    dynamic_state: &Mutex<LasmDynamicResponseState>,
    db_records_adapter: LasmDbRecordsAdapter,
    db: i64,
) -> Result<i64, LasmLockedOperationError> {
    let mut state = dynamic_state
        .lock()
        .map_err(|_| LasmLockedOperationError::StateUnavailable)?;
    if state.db_records_adapter != db_records_adapter {
        return Err(LasmLockedOperationError::AdapterMismatch);
    }
    let Some(tx) = allocate_lasm_db_tx_handle(&mut state, db) else {
        return Err(LasmLockedOperationError::Capacity {
            max_handles: state.db_tx_max_handles,
        });
    };
    Ok(tx)
}

pub(crate) fn resolve_lasm_exec_tx_state_bindings_locked(
    dynamic_state: &Mutex<LasmDynamicResponseState>,
    db_records_adapter: LasmDbRecordsAdapter,
    tx_source: &LasmExecTxSource,
) -> Result<LasmResolvedExecTxStateBindings, LasmLockedOperationError> {
    let mut state = dynamic_state
        .lock()
        .map_err(|_| LasmLockedOperationError::StateUnavailable)?;
    if state.db_records_adapter != db_records_adapter {
        return Err(LasmLockedOperationError::AdapterMismatch);
    }
    match tx_source {
        LasmExecTxSource::AllocateFromDb(db_value) => {
            let Some(tx_value) = allocate_lasm_db_tx_handle(&mut state, *db_value) else {
                return Err(LasmLockedOperationError::Capacity {
                    max_handles: state.db_tx_max_handles,
                });
            };
            if let Some(tx_state) = state.db_tx_handles.get_mut(&tx_value) {
                tx_state.in_use = true;
            }
            Ok(LasmResolvedExecTxStateBindings {
                db: *db_value,
                tx: tx_value,
                tx_active: false,
                allocated_tx_handle: Some(tx_value),
            })
        }
        LasmExecTxSource::ExistingTx(tx_value) => {
            let Some(tx_state) = state.db_tx_handles.get_mut(tx_value) else {
                return Err(LasmLockedOperationError::InvalidHandle);
            };
            if tx_state.in_use {
                return Err(LasmLockedOperationError::ConflictInUse);
            }
            tx_state.in_use = true;
            Ok(LasmResolvedExecTxStateBindings {
                db: tx_state.db,
                tx: *tx_value,
                tx_active: tx_state.active,
                allocated_tx_handle: None,
            })
        }
    }
}

pub(crate) fn run_lasm_postgres_query_one_unlocked_operation(
    dynamic_state: &Mutex<LasmDynamicResponseState>,
    db: i64,
    template: &str,
    params: &str,
    prepared_params: &LasmPreparedDbOperationParams,
) -> Result<LasmUnlockedPostgresQueryOneSuccess, LasmUnlockedPostgresOperationError> {
    let LasmPreparedDbOperationParams::Postgres {
        template: postgres_template,
        params: postgres_params,
    } = prepared_params
    else {
        return Err(LasmUnlockedPostgresOperationError::PreparationMismatch);
    };
    let config = {
        let state = dynamic_state
            .lock()
            .map_err(|_| LasmUnlockedPostgresOperationError::StateUnavailable)?;
        if state.db_records_adapter != LasmDbRecordsAdapter::Postgres {
            return Err(LasmUnlockedPostgresOperationError::AdapterMismatch);
        }
        build_lasm_postgres_thread_local_config(&state)
            .map_err(LasmUnlockedPostgresOperationError::Runtime)?
    };
    let row_object = match run_lasm_postgres_query_one_thread_local(
        &config,
        postgres_template.as_str(),
        postgres_params.as_slice(),
    ) {
        Ok(Some(value)) => value,
        Ok(None) => return Err(LasmUnlockedPostgresOperationError::NotFound),
        Err(message) => return Err(LasmUnlockedPostgresOperationError::Runtime(message)),
    };
    if !lasm_db_records_capture_enabled() {
        return Ok(LasmUnlockedPostgresQueryOneSuccess {
            record: allocate_lasm_db_ephemeral_record("queryOne", db, template, params, 0, 1),
            row_object,
        });
    }
    let (record, compaction_snapshot) = {
        let mut state = dynamic_state
            .lock()
            .map_err(|_| LasmUnlockedPostgresOperationError::StateUnavailable)?;
        if state.db_records_adapter != LasmDbRecordsAdapter::Postgres {
            return Err(LasmUnlockedPostgresOperationError::AdapterMismatch);
        }
        let record =
            allocate_lasm_db_runtime_record(&mut state, "queryOne", db, template, params, 0, 1);
        append_lasm_db_record_in_memory_with_compaction_snapshot(&mut state, record)
    };
    persist_lasm_db_record_after_unlock(
        LasmDbRecordsAdapter::Postgres,
        &config,
        &record,
        compaction_snapshot,
    );
    Ok(LasmUnlockedPostgresQueryOneSuccess { record, row_object })
}

pub(crate) fn run_lasm_non_postgres_query_one_locked_operation(
    dynamic_state: &Mutex<LasmDynamicResponseState>,
    db_records_adapter: LasmDbRecordsAdapter,
    db: i64,
    template: &str,
    params: &str,
    row_schema: i64,
    prepared_params: &LasmPreparedDbOperationParams,
) -> Result<LasmLockedQueryOneSuccess, LasmLockedQueryOneOperationError> {
    let mut state = dynamic_state
        .lock()
        .map_err(|_| LasmLockedQueryOneOperationError::StateUnavailable)?;
    if state.db_records_adapter != db_records_adapter
        || db_records_adapter == LasmDbRecordsAdapter::Postgres
    {
        return Err(LasmLockedQueryOneOperationError::AdapterMismatch);
    }
    let row_object = match run_lasm_db_query_one_operation(
        &mut state,
        db_records_adapter,
        db,
        template,
        params,
        row_schema,
        prepared_params,
    ) {
        Ok(LasmDbQueryOneOperationResult::Sqlite { row })
        | Ok(LasmDbQueryOneOperationResult::RecordsLog { row }) => row,
        Err(LasmDbQueryOneOperationError::NotFound) => {
            return Err(LasmLockedQueryOneOperationError::NotFound);
        }
        Err(LasmDbQueryOneOperationError::PreparationMismatch) => {
            return Err(LasmLockedQueryOneOperationError::PreparationMismatch);
        }
        Err(LasmDbQueryOneOperationError::Runtime(message)) => {
            return Err(LasmLockedQueryOneOperationError::Runtime(message));
        }
    };
    let record =
        allocate_lasm_db_runtime_record(&mut state, "queryOne", db, template, params, 0, 1);
    persist_lasm_db_record_with_capacity_guard(&mut state, &record);
    Ok(LasmLockedQueryOneSuccess { record, row_object })
}

pub(crate) fn run_lasm_query_one_operation_with_adapter(
    dynamic_state: &Mutex<LasmDynamicResponseState>,
    db_records_adapter: LasmDbRecordsAdapter,
    db: i64,
    template: &str,
    params: &str,
    row_schema: i64,
    prepared_params: &LasmPreparedDbOperationParams,
) -> Result<LasmUnifiedQueryOneSuccess, LasmUnifiedQueryOneOperationError> {
    if db_records_adapter == LasmDbRecordsAdapter::Postgres {
        return run_lasm_postgres_query_one_unlocked_operation(
            dynamic_state,
            db,
            template,
            params,
            prepared_params,
        )
        .map(|success| LasmUnifiedQueryOneSuccess {
            record: success.record,
            row_object: success.row_object,
        })
        .map_err(|error| match error {
            LasmUnlockedPostgresOperationError::StateUnavailable => {
                LasmUnifiedQueryOneOperationError::StateUnavailable
            }
            LasmUnlockedPostgresOperationError::AdapterMismatch => {
                LasmUnifiedQueryOneOperationError::AdapterMismatch
            }
            LasmUnlockedPostgresOperationError::PreparationMismatch => {
                LasmUnifiedQueryOneOperationError::PreparationMismatch
            }
            LasmUnlockedPostgresOperationError::NotFound => {
                LasmUnifiedQueryOneOperationError::NotFound
            }
            LasmUnlockedPostgresOperationError::Runtime(message) => {
                LasmUnifiedQueryOneOperationError::Runtime(message)
            }
        });
    }

    run_lasm_non_postgres_query_one_locked_operation(
        dynamic_state,
        db_records_adapter,
        db,
        template,
        params,
        row_schema,
        prepared_params,
    )
    .map(|success| LasmUnifiedQueryOneSuccess {
        record: success.record,
        row_object: success.row_object,
    })
    .map_err(|error| match error {
        LasmLockedQueryOneOperationError::StateUnavailable => {
            LasmUnifiedQueryOneOperationError::StateUnavailable
        }
        LasmLockedQueryOneOperationError::AdapterMismatch => {
            LasmUnifiedQueryOneOperationError::AdapterMismatch
        }
        LasmLockedQueryOneOperationError::NotFound => LasmUnifiedQueryOneOperationError::NotFound,
        LasmLockedQueryOneOperationError::PreparationMismatch => {
            LasmUnifiedQueryOneOperationError::PreparationMismatch
        }
        LasmLockedQueryOneOperationError::Runtime(message) => {
            LasmUnifiedQueryOneOperationError::Runtime(message)
        }
    })
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn run_lasm_postgres_exec_tx_unlocked_operation(
    dynamic_state: &Mutex<LasmDynamicResponseState>,
    db: i64,
    tx: i64,
    tx_active: bool,
    allocated_tx_handle: Option<i64>,
    keep_allocated_tx_handle: bool,
    template: &str,
    params: &str,
    prepared_params: &LasmPreparedDbOperationParams,
) -> Result<LasmUnlockedPostgresExecTxSuccess, LasmUnlockedPostgresExecTxOperationError> {
    let LasmPreparedDbOperationParams::Postgres {
        template: postgres_template,
        params: postgres_params,
    } = prepared_params
    else {
        return Err(LasmUnlockedPostgresExecTxOperationError::PreparationMismatch);
    };
    let (config, existing_tx_client) = {
        let mut state = dynamic_state
            .lock()
            .map_err(|_| LasmUnlockedPostgresExecTxOperationError::StateUnavailable)?;
        if state.db_records_adapter != LasmDbRecordsAdapter::Postgres {
            return Err(LasmUnlockedPostgresExecTxOperationError::AdapterMismatch);
        }
        let config = match build_lasm_postgres_thread_local_config(&state) {
            Ok(config) => config,
            Err(message) => {
                if keep_allocated_tx_handle {
                    if let Some(tx_state) = state.db_tx_handles.get_mut(&tx) {
                        tx_state.active = false;
                        tx_state.in_use = false;
                    }
                } else if allocated_tx_handle.is_some() {
                    drop_lasm_db_tx_handle(&mut state, tx);
                } else if let Some(tx_state) = state.db_tx_handles.get_mut(&tx) {
                    tx_state.in_use = false;
                }
                return Err(LasmUnlockedPostgresExecTxOperationError::Runtime(message));
            }
        };
        let existing_tx_client = take_lasm_postgres_tx_client_if_present(&mut state, tx);
        (config, existing_tx_client)
    };
    let client_result = run_lasm_postgres_exec_tx_client_operation(
        &config,
        tx,
        tx_active,
        keep_allocated_tx_handle,
        postgres_template.as_str(),
        postgres_params.as_slice(),
        existing_tx_client,
    );
    let LasmPostgresExecTxClientSuccess {
        affected_rows,
        active: retained_tx_active,
        retained_client,
    } = match client_result {
        Ok(value) => value,
        Err(error) => {
            let mut state = dynamic_state
                .lock()
                .map_err(|_| LasmUnlockedPostgresExecTxOperationError::StateUnavailable)?;
            if let Some(tx_state) = state.db_tx_handles.get_mut(&tx) {
                tx_state.active = false;
                tx_state.in_use = false;
            }
            if keep_allocated_tx_handle {
                if let Some(tx_state) = state.db_tx_handles.get_mut(&tx) {
                    tx_state.active = false;
                }
            } else {
                drop_lasm_db_tx_handle(&mut state, tx);
            }
            return Err(LasmUnlockedPostgresExecTxOperationError::Runtime(
                error.message,
            ));
        }
    };
    let mut retained_client = retained_client;
    let should_finalize_postgres_tx = retained_client.is_none();
    if !lasm_db_records_capture_enabled() {
        let mut state = dynamic_state.lock().map_err(|_| {
            if let Some(client) = retained_client.take() {
                discard_lasm_postgres_tx_client(&config, client);
            }
            LasmUnlockedPostgresExecTxOperationError::StateUnavailable
        })?;
        if state.db_records_adapter != LasmDbRecordsAdapter::Postgres {
            if let Some(client) = retained_client.take() {
                discard_lasm_postgres_tx_client(&config, client);
            }
            return Err(LasmUnlockedPostgresExecTxOperationError::AdapterMismatch);
        }
        if should_finalize_postgres_tx {
            drop_lasm_db_tx_handle(&mut state, tx);
        } else {
            if let Some(tx_state) = state.db_tx_handles.get_mut(&tx) {
                tx_state.active = retained_tx_active;
                tx_state.in_use = false;
            }
            if let Some(client) = retained_client.take() {
                put_lasm_postgres_tx_client(&mut state, tx, client);
            }
        }
        return Ok(LasmUnlockedPostgresExecTxSuccess {
            record: allocate_lasm_db_ephemeral_record(
                "execTx",
                db,
                template,
                params,
                tx,
                affected_rows,
            ),
        });
    }
    let (record, compaction_snapshot) = {
        let mut state = dynamic_state.lock().map_err(|_| {
            if let Some(client) = retained_client.take() {
                discard_lasm_postgres_tx_client(&config, client);
            }
            LasmUnlockedPostgresExecTxOperationError::StateUnavailable
        })?;
        if state.db_records_adapter != LasmDbRecordsAdapter::Postgres {
            if let Some(client) = retained_client.take() {
                discard_lasm_postgres_tx_client(&config, client);
            }
            return Err(LasmUnlockedPostgresExecTxOperationError::AdapterMismatch);
        }
        if should_finalize_postgres_tx {
            drop_lasm_db_tx_handle(&mut state, tx);
        } else {
            if let Some(tx_state) = state.db_tx_handles.get_mut(&tx) {
                tx_state.active = retained_tx_active;
                tx_state.in_use = false;
            }
            if let Some(client) = retained_client.take() {
                put_lasm_postgres_tx_client(&mut state, tx, client);
            }
        }
        let record = allocate_lasm_db_runtime_record(
            &mut state,
            "execTx",
            db,
            template,
            params,
            tx,
            affected_rows,
        );
        append_lasm_db_record_in_memory_with_compaction_snapshot(&mut state, record)
    };
    persist_lasm_db_record_after_unlock(
        LasmDbRecordsAdapter::Postgres,
        &config,
        &record,
        compaction_snapshot,
    );
    Ok(LasmUnlockedPostgresExecTxSuccess { record })
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn run_lasm_non_postgres_exec_tx_locked_operation(
    dynamic_state: &Mutex<LasmDynamicResponseState>,
    db_records_adapter: LasmDbRecordsAdapter,
    db: i64,
    tx: i64,
    tx_active: bool,
    allocated_tx_handle: Option<i64>,
    keep_allocated_tx_handle: bool,
    template: &str,
    params: &str,
    prepared_params: &LasmPreparedDbOperationParams,
) -> Result<LasmLockedExecTxSuccess, LasmLockedExecTxOperationError> {
    let mut state = dynamic_state
        .lock()
        .map_err(|_| LasmLockedExecTxOperationError::StateUnavailable)?;
    if state.db_records_adapter != db_records_adapter
        || db_records_adapter == LasmDbRecordsAdapter::Postgres
    {
        return Err(LasmLockedExecTxOperationError::AdapterMismatch);
    }
    let operation_result = match run_lasm_db_exec_tx_operation(
        &mut state,
        db_records_adapter,
        tx,
        tx_active,
        template,
        prepared_params,
    ) {
        Ok(value) => value,
        Err(error) => {
            if error.tx_started {
                let _ = run_lasm_db_tx_rollback(&mut state, db_records_adapter, tx);
                if let Some(tx_state) = state.db_tx_handles.get_mut(&tx) {
                    tx_state.active = false;
                    tx_state.in_use = false;
                }
                if !keep_allocated_tx_handle {
                    drop_lasm_db_tx_handle(&mut state, tx);
                }
            } else if let Some(tx_state) = state.db_tx_handles.get_mut(&tx) {
                tx_state.in_use = false;
            }
            if let Some(tx_handle) = allocated_tx_handle {
                if keep_allocated_tx_handle {
                    if let Some(tx_state) = state.db_tx_handles.get_mut(&tx_handle) {
                        tx_state.active = false;
                        tx_state.in_use = false;
                    }
                } else {
                    drop_lasm_db_tx_handle(&mut state, tx_handle);
                }
            }
            return Err(LasmLockedExecTxOperationError::Runtime(error.message));
        }
    };

    let (affected_rows, tx_started, should_commit_tx) = match operation_result {
        LasmDbExecTxOperationResult::Sqlite {
            affected_rows,
            tx_started,
        } => (affected_rows, tx_started, true),
        LasmDbExecTxOperationResult::RecordsLog { affected_rows } => (affected_rows, false, false),
    };

    if should_commit_tx && tx_started && !keep_allocated_tx_handle {
        if let Err(message) = run_lasm_db_tx_commit(&mut state, db_records_adapter, tx) {
            drop_lasm_db_tx_handle(&mut state, tx);
            return Err(LasmLockedExecTxOperationError::Runtime(message));
        }
        drop_lasm_db_tx_handle(&mut state, tx);
    }
    if let Some(tx_handle) = allocated_tx_handle {
        if !keep_allocated_tx_handle {
            drop_lasm_db_tx_handle(&mut state, tx_handle);
        } else if let Some(tx_state) = state.db_tx_handles.get_mut(&tx_handle) {
            tx_state.active = tx_started || tx_active;
            tx_state.in_use = false;
        }
    } else if let Some(tx_state) = state.db_tx_handles.get_mut(&tx) {
        if keep_allocated_tx_handle {
            tx_state.active = tx_started || tx_active;
        }
        tx_state.in_use = false;
    }
    let record = allocate_lasm_db_runtime_record(
        &mut state,
        "execTx",
        db,
        template,
        params,
        tx,
        affected_rows,
    );
    persist_lasm_db_record_with_capacity_guard(&mut state, &record);
    Ok(LasmLockedExecTxSuccess { record })
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn run_lasm_exec_tx_operation_with_adapter(
    dynamic_state: &Mutex<LasmDynamicResponseState>,
    db_records_adapter: LasmDbRecordsAdapter,
    db: i64,
    tx: i64,
    tx_active: bool,
    allocated_tx_handle: Option<i64>,
    keep_allocated_tx_handle: bool,
    template: &str,
    params: &str,
    prepared_params: &LasmPreparedDbOperationParams,
) -> Result<LasmUnifiedExecTxSuccess, LasmUnifiedExecTxOperationError> {
    if db_records_adapter == LasmDbRecordsAdapter::Postgres {
        return run_lasm_postgres_exec_tx_unlocked_operation(
            dynamic_state,
            db,
            tx,
            tx_active,
            allocated_tx_handle,
            keep_allocated_tx_handle,
            template,
            params,
            prepared_params,
        )
        .map(|success| LasmUnifiedExecTxSuccess {
            record: success.record,
        })
        .map_err(|error| match error {
            LasmUnlockedPostgresExecTxOperationError::StateUnavailable => {
                LasmUnifiedExecTxOperationError::StateUnavailable
            }
            LasmUnlockedPostgresExecTxOperationError::AdapterMismatch => {
                LasmUnifiedExecTxOperationError::AdapterMismatch
            }
            LasmUnlockedPostgresExecTxOperationError::PreparationMismatch => {
                LasmUnifiedExecTxOperationError::PreparationMismatch
            }
            LasmUnlockedPostgresExecTxOperationError::Runtime(message) => {
                LasmUnifiedExecTxOperationError::Runtime(message)
            }
        });
    }

    run_lasm_non_postgres_exec_tx_locked_operation(
        dynamic_state,
        db_records_adapter,
        db,
        tx,
        tx_active,
        allocated_tx_handle,
        keep_allocated_tx_handle,
        template,
        params,
        prepared_params,
    )
    .map(|success| LasmUnifiedExecTxSuccess {
        record: success.record,
    })
    .map_err(|error| match error {
        LasmLockedExecTxOperationError::StateUnavailable => {
            LasmUnifiedExecTxOperationError::StateUnavailable
        }
        LasmLockedExecTxOperationError::AdapterMismatch => {
            LasmUnifiedExecTxOperationError::AdapterMismatch
        }
        LasmLockedExecTxOperationError::Runtime(message) => {
            LasmUnifiedExecTxOperationError::Runtime(message)
        }
    })
}

pub(crate) fn run_lasm_db_query_one_operation(
    state: &mut LasmDynamicResponseState,
    db_records_adapter: LasmDbRecordsAdapter,
    db: i64,
    template: &str,
    params: &str,
    row_schema: i64,
    prepared_params: &LasmPreparedDbOperationParams,
) -> Result<LasmDbQueryOneOperationResult, LasmDbQueryOneOperationError> {
    match db_records_adapter {
        LasmDbRecordsAdapter::Postgres => Err(LasmDbQueryOneOperationError::Runtime(
            "internal postgres queryOne must run through the unlocked thread-local path"
                .to_string(),
        )),
        LasmDbRecordsAdapter::Sqlite => {
            let LasmPreparedDbOperationParams::Sqlite { params } = prepared_params else {
                return Err(LasmDbQueryOneOperationError::PreparationMismatch);
            };
            let row = run_lasm_sqlite_query_one(state, template, params)
                .map_err(LasmDbQueryOneOperationError::Runtime)?;
            match row {
                Some(value) => Ok(LasmDbQueryOneOperationResult::Sqlite { row: value }),
                None => Err(LasmDbQueryOneOperationError::NotFound),
            }
        }
        LasmDbRecordsAdapter::RecordsLog => {
            let Some(matched_source_record) =
                find_lasm_records_log_latest_match(state, db, template, params)
            else {
                return Err(LasmDbQueryOneOperationError::NotFound);
            };
            let row =
                build_lasm_records_log_query_one_row_object(&matched_source_record, row_schema);
            Ok(LasmDbQueryOneOperationResult::RecordsLog { row })
        }
    }
}
