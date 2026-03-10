use super::types::{
    LasmLockedExecTxOperationError, LasmLockedQueryOneOperationError,
    LasmPreparedDbOperationParams, LasmUnifiedExecOperationError, LasmUnifiedExecSuccess,
    LasmUnifiedExecTxOperationError, LasmUnifiedExecTxSuccess, LasmUnifiedQueryOneOperationError,
    LasmUnifiedQueryOneSuccess,
};
use super::LasmLockedOperationError;
use super::{
    run_lasm_non_postgres_exec_locked_operation, run_lasm_non_postgres_exec_tx_locked_operation,
    run_lasm_non_postgres_query_one_locked_operation, run_lasm_postgres_exec_tx_unlocked_operation,
    run_lasm_postgres_exec_unlocked_operation, run_lasm_postgres_query_one_unlocked_operation,
    LasmUnlockedPostgresExecTxOperationError, LasmUnlockedPostgresOperationError,
};
use crate::{LasmDbRecordsAdapter, LasmDynamicResponseState};
use std::sync::Mutex;

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
