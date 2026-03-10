use super::config::build_lasm_postgres_thread_local_config;
use super::drop_lasm_db_tx_handle;
use super::lasm_db_records_capture_enabled;
use super::persist_lasm_db_record_after_unlock;
use super::records::{
    allocate_lasm_db_ephemeral_record, allocate_lasm_db_runtime_record,
    append_lasm_db_record_in_memory_with_compaction_snapshot,
};
use super::LasmPreparedDbOperationParams;
use crate::lasm_db_runtime_postgres::{
    connect_lasm_postgres_tx_client, discard_lasm_postgres_tx_client, put_lasm_postgres_tx_client,
    return_lasm_postgres_tx_client_to_pool, run_lasm_postgres_exec_thread_local,
    run_lasm_postgres_exec_tx_commit_on_client, run_lasm_postgres_exec_tx_on_client,
    run_lasm_postgres_exec_tx_one_shot_on_client, run_lasm_postgres_exec_tx_rollback_on_client,
    run_lasm_postgres_query_one_thread_local, take_lasm_postgres_tx_client_if_present,
    LasmPostgresParam, LasmPostgresThreadLocalClient, LasmPostgresThreadLocalConfig,
};
use crate::{LasmDbRecord, LasmDbRecordsAdapter, LasmDynamicResponseState};
use std::sync::Mutex;

pub(crate) struct LasmPostgresExecTxClientSuccess {
    pub(crate) affected_rows: u64,
    pub(crate) active: bool,
    pub(crate) retained_client: Option<LasmPostgresThreadLocalClient>,
}

pub(crate) struct LasmPostgresExecTxClientError {
    pub(crate) message: String,
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

pub(crate) struct LasmUnlockedPostgresQueryOneSuccess {
    pub(crate) record: LasmDbRecord,
    pub(crate) row_object: serde_json::Value,
}

pub(crate) struct LasmUnlockedPostgresExecTxSuccess {
    pub(crate) record: LasmDbRecord,
}

pub(crate) enum LasmUnlockedPostgresExecTxOperationError {
    StateUnavailable,
    AdapterMismatch,
    PreparationMismatch,
    Runtime(String),
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
        let mut state = dynamic_state
            .lock()
            .map_err(|_| LasmUnlockedPostgresOperationError::StateUnavailable)?;
        if state.db_records_adapter != LasmDbRecordsAdapter::Postgres {
            return Err(LasmUnlockedPostgresOperationError::AdapterMismatch);
        }
        build_lasm_postgres_thread_local_config(&mut state)
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
        let mut state = dynamic_state
            .lock()
            .map_err(|_| LasmUnlockedPostgresOperationError::StateUnavailable)?;
        if state.db_records_adapter != LasmDbRecordsAdapter::Postgres {
            return Err(LasmUnlockedPostgresOperationError::AdapterMismatch);
        }
        build_lasm_postgres_thread_local_config(&mut state)
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
        let config = match build_lasm_postgres_thread_local_config(&mut state) {
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
