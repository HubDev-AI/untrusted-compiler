use crate::lasm_db_config::LASM_DB_POSTGRES_DSN_CONFIG_ERROR_MESSAGE;
use crate::lasm_db_runtime_postgres::{
    parse_lasm_postgres_query_template_and_params,
    parse_lasm_postgres_query_template_and_params_value, run_lasm_postgres_exec_thread_local,
    run_lasm_postgres_exec_tx, run_lasm_postgres_exec_tx_commit,
    run_lasm_postgres_exec_tx_rollback, run_lasm_postgres_query_one_thread_local,
    LasmPostgresParam, LasmPostgresThreadLocalConfig,
};
use crate::lasm_db_runtime_sqlite::{
    parse_lasm_sqlite_query_params, parse_lasm_sqlite_query_params_value, run_lasm_sqlite_exec,
    run_lasm_sqlite_exec_tx, run_lasm_sqlite_exec_tx_commit, run_lasm_sqlite_exec_tx_rollback,
    run_lasm_sqlite_query_one, LasmSqliteQueryParams,
};
use crate::{LasmDbRecordsAdapter, LasmDynamicResponseState};
use std::collections::BTreeSet;
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
    Postgres {
        config: LasmPostgresThreadLocalConfig,
        affected_rows: u64,
    },
    Sqlite {
        affected_rows: u64,
    },
    RecordsLog {
        affected_rows: u64,
    },
}

pub(crate) enum LasmDbExecTxOperationResult {
    Postgres {
        config: LasmPostgresThreadLocalConfig,
        affected_rows: u64,
        tx_started: bool,
    },
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

pub(crate) enum LasmDbQueryOneOperationResult {
    Postgres {
        config: LasmPostgresThreadLocalConfig,
        row: serde_json::Value,
    },
    Sqlite {
        row: serde_json::Value,
    },
}

#[derive(Debug)]
pub(crate) enum LasmDbQueryOneOperationError {
    PreparationMismatch,
    NotFound,
    Runtime(String),
}

pub(crate) fn build_lasm_postgres_thread_local_config(
    state: &LasmDynamicResponseState,
) -> Result<LasmPostgresThreadLocalConfig, String> {
    let dsn = state
        .db_records_postgres_dsn
        .as_deref()
        .ok_or_else(|| LASM_DB_POSTGRES_DSN_CONFIG_ERROR_MESSAGE.to_string())?;
    Ok(LasmPostgresThreadLocalConfig {
        dsn: dsn.to_string(),
        tls_mode: state.db_postgres_tls_mode,
        statement_timeout_ms: state.db_postgres_statement_timeout_ms.max(1),
        lock_timeout_ms: state.db_postgres_lock_timeout_ms.max(1),
        connect_timeout_ms: state.db_postgres_connect_timeout_ms.max(1),
        db_postgres_statement_cache_max: state.db_postgres_statement_cache_max,
        db_postgres_placeholder_cache_max: state.db_postgres_placeholder_cache_max,
        retryable_conflict_retry_max: state.db_postgres_retryable_conflict_retry_max,
    })
}

pub(crate) fn parse_lasm_db_template_and_params(
    adapter: LasmDbRecordsAdapter,
    template: &str,
    params: &str,
    parsed_params: Option<&serde_json::Value>,
) -> Result<LasmPreparedDbOperationParams, String> {
    let parsed_params = parsed_params.cloned();
    match adapter {
        LasmDbRecordsAdapter::Postgres => {
            let result = if let Some(parsed) = parsed_params {
                parse_lasm_postgres_query_template_and_params_value(template, &parsed)
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
                parse_lasm_sqlite_query_params_value(&parsed)
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
        LasmDbRecordsAdapter::Postgres => {
            let LasmPreparedDbOperationParams::Postgres { template, params } = prepared_params
            else {
                return Err("internal db operation preparation mismatch".to_string());
            };
            let config = build_lasm_postgres_thread_local_config(state)?;
            let affected_rows =
                run_lasm_postgres_exec_thread_local(&config, template, params.as_slice())?;
            Ok(LasmDbExecOperationResult::Postgres {
                config,
                affected_rows,
            })
        }
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
        LasmDbRecordsAdapter::Postgres => {
            let LasmPreparedDbOperationParams::Postgres {
                template: parsed_template,
                params,
            } = prepared_params
            else {
                return to_error(
                    "internal db operation preparation mismatch".to_string(),
                    false,
                );
            };
            let config = match build_lasm_postgres_thread_local_config(state) {
                Ok(config) => config,
                Err(message) => return to_error(message, false),
            };
            match run_lasm_postgres_exec_tx(
                state,
                tx,
                parsed_template,
                params.as_slice(),
                tx_active,
            ) {
                Ok((affected_rows, tx_started)) => {
                    if tx_started {
                        if let Some(tx_state) = state.db_tx_handles.get_mut(&tx) {
                            tx_state.active = true;
                        }
                    }
                    Ok(LasmDbExecTxOperationResult::Postgres {
                        config,
                        affected_rows,
                        tx_started,
                    })
                }
                Err((message, tx_started)) => to_error(message, tx_started),
            }
        }
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

pub(crate) fn run_lasm_db_query_one_operation(
    state: &mut LasmDynamicResponseState,
    db_records_adapter: LasmDbRecordsAdapter,
    template: &str,
    prepared_params: &LasmPreparedDbOperationParams,
) -> Result<LasmDbQueryOneOperationResult, LasmDbQueryOneOperationError> {
    match db_records_adapter {
        LasmDbRecordsAdapter::Postgres => {
            let LasmPreparedDbOperationParams::Postgres {
                template: parsed_template,
                params,
            } = prepared_params
            else {
                return Err(LasmDbQueryOneOperationError::PreparationMismatch);
            };
            let config = build_lasm_postgres_thread_local_config(state)
                .map_err(LasmDbQueryOneOperationError::Runtime)?;
            let row = run_lasm_postgres_query_one_thread_local(
                &config,
                parsed_template,
                params.as_slice(),
            )
            .map_err(LasmDbQueryOneOperationError::Runtime)?;
            match row {
                Some(value) => Ok(LasmDbQueryOneOperationResult::Postgres { config, row: value }),
                None => Err(LasmDbQueryOneOperationError::NotFound),
            }
        }
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
        LasmDbRecordsAdapter::RecordsLog => Err(LasmDbQueryOneOperationError::Runtime(
            "records log queryOne helper is handled by dispatch".to_string(),
        )),
    }
}

pub(crate) fn cleanup_lasm_internal_db_sequence_tx_handles(
    dynamic_state: &Mutex<LasmDynamicResponseState>,
    db_records_adapter: LasmDbRecordsAdapter,
    handles: impl IntoIterator<Item = i64>,
    operation_succeeded: bool,
) {
    let Ok(mut state) = dynamic_state.lock() else {
        return;
    };
    let mut consumed_handles = BTreeSet::new();
    for handle in handles {
        if !consumed_handles.insert(handle) {
            continue;
        }
        let Some(tx_state) = state.db_tx_handles.get(&handle).copied() else {
            continue;
        };
        if !tx_state.active {
            state.db_tx_handles.remove(&handle);
            continue;
        }
        match db_records_adapter {
            LasmDbRecordsAdapter::Sqlite => {
                if operation_succeeded {
                    if let Err(_message) = run_lasm_sqlite_exec_tx_commit(&mut state, handle) {
                        state.db_tx_handles.remove(&handle);
                        continue;
                    }
                } else if let Err(_message) = run_lasm_sqlite_exec_tx_rollback(&mut state, handle) {
                    state.db_tx_handles.remove(&handle);
                    continue;
                }
            }
            LasmDbRecordsAdapter::Postgres => {
                if operation_succeeded {
                    let _ = run_lasm_postgres_exec_tx_commit(&mut state, handle);
                } else {
                    let _ = run_lasm_postgres_exec_tx_rollback(&mut state, handle);
                }
            }
            LasmDbRecordsAdapter::RecordsLog => {}
        }
        state.db_tx_handles.remove(&handle);
    }
}
