use crate::lasm_db_adapter_state::{
    persist_lasm_dynamic_db_record_append, persist_lasm_dynamic_db_records_full_sync,
};
use crate::lasm_db_runtime_common::lasm_dynamic_postgres_client_mut;
use crate::lasm_db_runtime_postgres::{
    run_lasm_postgres_exec_tx_commit, run_lasm_postgres_exec_tx_rollback,
    LasmPostgresThreadLocalConfig,
};
use crate::lasm_db_runtime_postgres_persist::persist_lasm_postgres_record_after_unlock;
use crate::lasm_db_runtime_sqlite::{
    run_lasm_sqlite_exec_tx_commit, run_lasm_sqlite_exec_tx_rollback,
};
use crate::{LasmDbRecordsAdapter, LasmDynamicResponseState};
use std::collections::BTreeSet;
use std::sync::Mutex;

pub(crate) fn ensure_lasm_db_records_client_ready(
    state: &mut LasmDynamicResponseState,
    db_records_adapter: LasmDbRecordsAdapter,
) -> Result<(), String> {
    match db_records_adapter {
        LasmDbRecordsAdapter::Postgres => lasm_dynamic_postgres_client_mut(state).map(|_client| ()),
        LasmDbRecordsAdapter::Sqlite | LasmDbRecordsAdapter::RecordsLog => Ok(()),
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

pub(crate) fn run_lasm_db_tx_commit(
    state: &mut LasmDynamicResponseState,
    adapter: LasmDbRecordsAdapter,
    tx: i64,
) -> Result<(), String> {
    match adapter {
        LasmDbRecordsAdapter::Postgres => run_lasm_postgres_exec_tx_commit(state, tx),
        LasmDbRecordsAdapter::Sqlite => run_lasm_sqlite_exec_tx_commit(state, tx),
        LasmDbRecordsAdapter::RecordsLog => Ok(()),
    }
}

pub(crate) fn run_lasm_db_tx_rollback(
    state: &mut LasmDynamicResponseState,
    adapter: LasmDbRecordsAdapter,
    tx: i64,
) -> Result<(), String> {
    match adapter {
        LasmDbRecordsAdapter::Postgres => run_lasm_postgres_exec_tx_rollback(state, tx),
        LasmDbRecordsAdapter::Sqlite => run_lasm_sqlite_exec_tx_rollback(state, tx),
        LasmDbRecordsAdapter::RecordsLog => Ok(()),
    }
}

pub(crate) fn persist_lasm_db_record_after_unlock(
    db_records_adapter: LasmDbRecordsAdapter,
    config: &LasmPostgresThreadLocalConfig,
    record: &crate::LasmDbRecord,
    compaction_snapshot: Option<Vec<crate::LasmDbRecord>>,
) {
    match db_records_adapter {
        LasmDbRecordsAdapter::Postgres => {
            persist_lasm_postgres_record_after_unlock(config, record, compaction_snapshot)
        }
        LasmDbRecordsAdapter::Sqlite | LasmDbRecordsAdapter::RecordsLog => {}
    }
}

pub(crate) fn persist_lasm_db_record_append(
    state: &mut LasmDynamicResponseState,
    record: &crate::LasmDbRecord,
) -> Result<(), String> {
    persist_lasm_dynamic_db_record_append(state, record)
}

pub(crate) fn persist_lasm_db_records_full_sync(
    state: &mut LasmDynamicResponseState,
) -> Result<(), String> {
    persist_lasm_dynamic_db_records_full_sync(state)
}
