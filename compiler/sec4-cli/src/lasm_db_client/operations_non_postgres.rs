use super::drop_lasm_db_tx_handle;
use super::operations::{
    LasmDbExecOperationResult, LasmDbExecTxError, LasmDbExecTxOperationResult,
    LasmDbQueryOneOperationError, LasmDbQueryOneOperationResult, LasmLockedExecSuccess,
    LasmLockedExecTxOperationError, LasmLockedExecTxSuccess, LasmLockedQueryOneOperationError,
    LasmLockedQueryOneSuccess, LasmPreparedDbOperationParams,
};
use super::records::{allocate_lasm_db_runtime_record, persist_lasm_db_record_with_capacity_guard};
use super::{run_lasm_db_tx_commit, run_lasm_db_tx_rollback, LasmLockedOperationError};
use crate::lasm_db_runtime_records_log::{
    build_lasm_records_log_query_one_row_object, find_lasm_records_log_latest_match,
};
use crate::lasm_db_runtime_sqlite::{
    run_lasm_sqlite_exec, run_lasm_sqlite_exec_tx, run_lasm_sqlite_query_one,
};
use crate::{LasmDbRecordsAdapter, LasmDynamicResponseState};
use std::sync::Mutex;

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
