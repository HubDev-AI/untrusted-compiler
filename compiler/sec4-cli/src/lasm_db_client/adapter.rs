use crate::lasm_db_adapter_state::{
    persist_lasm_dynamic_db_record_append, persist_lasm_dynamic_db_records_full_sync,
};
use crate::lasm_db_runtime_common::lasm_dynamic_postgres_client_mut;
use crate::lasm_db_runtime_postgres::{
    return_lasm_postgres_tx_client_to_pool, run_lasm_postgres_exec_tx_commit,
    run_lasm_postgres_exec_tx_commit_on_client, run_lasm_postgres_exec_tx_rollback,
    run_lasm_postgres_exec_tx_rollback_on_client, take_lasm_postgres_tx_client_if_present,
    LasmPostgresThreadLocalClient, LasmPostgresThreadLocalConfig,
};
use crate::lasm_db_runtime_postgres_persist::persist_lasm_postgres_record_after_unlock;
use crate::lasm_db_runtime_sqlite::{
    run_lasm_sqlite_exec_tx_commit, run_lasm_sqlite_exec_tx_rollback,
};
use crate::{LasmDbRecordsAdapter, LasmDynamicResponseState};
use std::collections::BTreeSet;
use std::env;
use std::sync::Mutex;
use std::sync::OnceLock;

const LASM_DB_RECORDS_PERSIST_ENABLED_ENV_KEYS: [&str; 2] = [
    "SEC4_DB_ALPHA_DB_RECORDS_PERSIST_ENABLED",
    "SEC4_RT_LASM_DB_RECORDS_PERSIST_ENABLED",
];
const LASM_DB_RECORDS_CAPTURE_ENABLED_ENV_KEYS: [&str; 2] = [
    "SEC4_DB_ALPHA_DB_RECORDS_CAPTURE_ENABLED",
    "SEC4_RT_LASM_DB_RECORDS_CAPTURE_ENABLED",
];

static LASM_DB_RECORDS_PERSIST_ENABLED: OnceLock<bool> = OnceLock::new();
static LASM_DB_RECORDS_CAPTURE_ENABLED: OnceLock<bool> = OnceLock::new();

pub(crate) fn lasm_db_records_persist_enabled() -> bool {
    *LASM_DB_RECORDS_PERSIST_ENABLED.get_or_init(|| {
        for key in LASM_DB_RECORDS_PERSIST_ENABLED_ENV_KEYS {
            let Ok(raw) = env::var(key) else {
                continue;
            };
            let normalized = raw.trim().to_ascii_lowercase();
            if normalized.is_empty() {
                continue;
            }
            return !matches!(normalized.as_str(), "0" | "false" | "no" | "off");
        }
        true
    })
}

pub(crate) fn lasm_db_records_capture_enabled() -> bool {
    *LASM_DB_RECORDS_CAPTURE_ENABLED.get_or_init(|| {
        for key in LASM_DB_RECORDS_CAPTURE_ENABLED_ENV_KEYS {
            let Ok(raw) = env::var(key) else {
                continue;
            };
            let normalized = raw.trim().to_ascii_lowercase();
            if normalized.is_empty() {
                continue;
            }
            return !matches!(normalized.as_str(), "0" | "false" | "no" | "off");
        }
        true
    })
}

struct PendingLasmPostgresSequenceTxCleanup {
    handle: i64,
    client: LasmPostgresThreadLocalClient,
}

pub(crate) fn ensure_lasm_db_records_client_ready(
    state: &mut LasmDynamicResponseState,
    db_records_adapter: LasmDbRecordsAdapter,
) -> Result<(), String> {
    match db_records_adapter {
        LasmDbRecordsAdapter::Postgres if !lasm_db_records_persist_enabled() => Ok(()),
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
    let mut pending_postgres = Vec::new();
    let mut postgres_config = None;
    let Ok(mut state) = dynamic_state.lock() else {
        return;
    };
    if db_records_adapter == LasmDbRecordsAdapter::Postgres {
        postgres_config =
            crate::lasm_db_client::build_lasm_postgres_thread_local_config(&state).ok();
    }
    let mut consumed_handles = BTreeSet::new();
    for handle in handles {
        if !consumed_handles.insert(handle) {
            continue;
        }
        let Some(tx_state) = state.db_tx_handles.get(&handle).copied() else {
            continue;
        };
        if !tx_state.active {
            drop_lasm_db_tx_handle(&mut state, handle);
            continue;
        }
        match db_records_adapter {
            LasmDbRecordsAdapter::Sqlite => {
                if operation_succeeded {
                    if let Err(_message) = run_lasm_sqlite_exec_tx_commit(&mut state, handle) {
                        drop_lasm_db_tx_handle(&mut state, handle);
                        continue;
                    }
                } else if let Err(_message) = run_lasm_sqlite_exec_tx_rollback(&mut state, handle) {
                    drop_lasm_db_tx_handle(&mut state, handle);
                    continue;
                }
            }
            LasmDbRecordsAdapter::Postgres => {
                if let Some(client) = take_lasm_postgres_tx_client_if_present(&mut state, handle) {
                    pending_postgres.push(PendingLasmPostgresSequenceTxCleanup { handle, client });
                    continue;
                }
                if operation_succeeded {
                    let _ = run_lasm_postgres_exec_tx_commit(&mut state, handle);
                } else {
                    let _ = run_lasm_postgres_exec_tx_rollback(&mut state, handle);
                }
            }
            LasmDbRecordsAdapter::RecordsLog => {}
        }
        drop_lasm_db_tx_handle(&mut state, handle);
    }
    drop(state);

    for pending in pending_postgres {
        let mut client = pending.client;
        let finalized = if operation_succeeded {
            run_lasm_postgres_exec_tx_commit_on_client(&mut client, pending.handle).is_ok()
        } else {
            run_lasm_postgres_exec_tx_rollback_on_client(&mut client, pending.handle).is_ok()
        };
        if finalized {
            if let Some(config) = postgres_config.as_ref() {
                return_lasm_postgres_tx_client_to_pool(config, client);
            }
        }
        let Ok(mut state) = dynamic_state.lock() else {
            return;
        };
        drop_lasm_db_tx_handle(&mut state, pending.handle);
    }
}

pub(crate) fn drop_lasm_db_tx_handle(state: &mut LasmDynamicResponseState, handle: i64) {
    state.db_tx_handles.remove(&handle);
    state.db_postgres_tx_clients.remove(&handle);
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
    if !lasm_db_records_persist_enabled() {
        return;
    }
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
    if !lasm_db_records_persist_enabled() {
        return Ok(());
    }
    persist_lasm_dynamic_db_record_append(state, record)
}

pub(crate) fn persist_lasm_db_records_full_sync(
    state: &mut LasmDynamicResponseState,
) -> Result<(), String> {
    if !lasm_db_records_persist_enabled() {
        return Ok(());
    }
    persist_lasm_dynamic_db_records_full_sync(state)
}
