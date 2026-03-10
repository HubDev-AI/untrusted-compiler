use super::LasmExecTxSource;
use crate::lasm_db_runtime_common::allocate_lasm_db_tx_handle;
use crate::{LasmDbRecordsAdapter, LasmDynamicResponseState};
use std::sync::Mutex;

pub(crate) struct LasmResolvedExecTxStateBindings {
    pub(crate) db: i64,
    pub(crate) tx: i64,
    pub(crate) tx_active: bool,
    pub(crate) allocated_tx_handle: Option<i64>,
}

pub(crate) enum LasmLockedOperationError {
    StateUnavailable,
    AdapterMismatch,
    Capacity { max_handles: usize },
    InvalidHandle,
    ConflictInUse,
    Runtime(String),
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
