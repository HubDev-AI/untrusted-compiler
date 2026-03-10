use crate::lasm_db_runtime_postgres::LasmPostgresParam;
use crate::lasm_db_runtime_sqlite::LasmSqliteQueryParams;
use crate::LasmDbRecord;

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

pub(crate) struct LasmLockedExecSuccess {
    pub(crate) record: LasmDbRecord,
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

pub(crate) enum LasmLockedQueryOneOperationError {
    StateUnavailable,
    AdapterMismatch,
    NotFound,
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
