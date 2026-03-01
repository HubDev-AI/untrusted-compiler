use crate::lasm_db_adapter_state::{
    persist_lasm_dynamic_db_record_append, persist_lasm_dynamic_db_records_full_sync,
};
use crate::lasm_db_records_response::apply_lasm_db_list_records_response_materialization;
use crate::lasm_db_runtime_common::{
    allocate_lasm_db_tx_handle, classify_lasm_db_runtime_error, is_lasm_valid_db_cap_handle,
    normalize_lasm_db_params_and_value, parse_lasm_positive_i64,
};
use crate::lasm_db_runtime_postgres::{
    parse_lasm_postgres_query_template_and_params,
    parse_lasm_postgres_query_template_and_params_value, run_lasm_postgres_exec_thread_local,
    run_lasm_postgres_exec_tx_thread_local, run_lasm_postgres_query_one_thread_local,
    LasmPostgresParam, LasmPostgresThreadLocalConfig,
};
use crate::lasm_db_runtime_postgres_persist::persist_lasm_postgres_record_after_unlock;
use crate::lasm_db_runtime_records_log::{
    build_lasm_records_log_query_one_row_object, find_lasm_records_log_latest_match,
};
use crate::lasm_db_runtime_sqlite::{
    parse_lasm_sqlite_query_params, parse_lasm_sqlite_query_params_value, run_lasm_sqlite_exec,
    run_lasm_sqlite_exec_tx, run_lasm_sqlite_query_one, LasmSqliteQueryParams,
};
use crate::{
    append_lasm_dynamic_db_record, lasm_db_record_to_json, lasm_error_envelope,
    lasm_internal_db_indexed_header, lasm_now_ms, set_lasm_json_response, LasmDbRecord,
    LasmDbRecordsAdapter, LasmDynamicResponseState, LasmRunRequest, LASM_INTERNAL_DB_HANDLE_HEADER,
    LASM_INTERNAL_DB_OP_COUNT_HEADER, LASM_INTERNAL_DB_OP_HEADER, LASM_INTERNAL_DB_OP_SEQUENCE_MAX,
    LASM_INTERNAL_DB_PARAMS_HEADER, LASM_INTERNAL_DB_ROW_SCHEMA_HEADER,
    LASM_INTERNAL_DB_TEMPLATE_HEADER, LASM_INTERNAL_DB_TX_DB_HEADER, LASM_INTERNAL_DB_TX_HEADER,
    LASM_INTERNAL_DB_TX_RESULT_HEADER, LASM_INTERNAL_DB_TX_SEQUENCE_RETAIN_HEADER,
};
use std::collections::BTreeMap;
use std::env;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock};

const LASM_DB_SQL_TEMPLATE_MAX_BYTES_ENV: &str = "SEC4_RT_LASM_DB_SQL_TEMPLATE_MAX_BYTES";
const LASM_DB_SQL_TEMPLATE_MAX_BYTES_DEFAULT: usize = 64 * 1024;
const LASM_DB_SQL_TEMPLATE_MAX_BYTES_MIN: usize = 256;
const LASM_DB_SQL_TEMPLATE_MAX_BYTES_MAX: usize = 4 * 1024 * 1024;
const LASM_DB_PARAMS_MAX_BYTES_ENV: &str = "SEC4_RT_LASM_DB_PARAMS_MAX_BYTES";
const LASM_DB_PARAMS_MAX_BYTES_DEFAULT: usize = 128 * 1024;
const LASM_DB_PARAMS_MAX_BYTES_MIN: usize = 256;
const LASM_DB_PARAMS_MAX_BYTES_MAX: usize = 8 * 1024 * 1024;
const LASM_DB_PARAMS_MAX_ENTRIES_ENV: &str = "SEC4_RT_LASM_DB_PARAMS_MAX_ENTRIES";
const LASM_DB_PARAMS_MAX_ENTRIES_DEFAULT: usize = 2048;
const LASM_DB_PARAMS_MAX_ENTRIES_MIN: usize = 1;
const LASM_DB_PARAMS_MAX_ENTRIES_MAX: usize = 65_536;
const LASM_DB_OP_SEQUENCE_MAX_ENV: &str = "SEC4_RT_LASM_DB_OP_SEQUENCE_MAX";
const LASM_DB_OP_SEQUENCE_MAX_DEFAULT: usize = LASM_INTERNAL_DB_OP_SEQUENCE_MAX;
const LASM_DB_OP_SEQUENCE_MAX_MIN: usize = 2;
const LASM_DB_OP_SEQUENCE_MAX_MAX: usize = 4096;
const LASM_DB_QUERY_ONE_ROW_MAX_BYTES_ENV: &str = "SEC4_RT_LASM_DB_QUERY_ONE_ROW_MAX_BYTES";
const LASM_DB_QUERY_ONE_ROW_MAX_BYTES_DEFAULT: usize = 1024 * 1024;
const LASM_DB_QUERY_ONE_ROW_MAX_BYTES_MIN: usize = 256;
const LASM_DB_QUERY_ONE_ROW_MAX_BYTES_MAX: usize = 16 * 1024 * 1024;
const LASM_DB_QUERY_ONE_ROW_MAX_COLUMNS_ENV: &str = "SEC4_RT_LASM_DB_QUERY_ONE_ROW_MAX_COLUMNS";
const LASM_DB_QUERY_ONE_ROW_MAX_COLUMNS_DEFAULT: usize = 1024;
const LASM_DB_QUERY_ONE_ROW_MAX_COLUMNS_MIN: usize = 1;
const LASM_DB_QUERY_ONE_ROW_MAX_COLUMNS_MAX: usize = 16_384;
const LASM_DB_RECORDS_COMPACTION_SYNC_DROPS_INTERVAL: u64 = 1024;
static LASM_DB_SQL_TEMPLATE_MAX_BYTES_RESOLVED: OnceLock<usize> = OnceLock::new();
static LASM_DB_PARAMS_MAX_BYTES_RESOLVED: OnceLock<usize> = OnceLock::new();
static LASM_DB_PARAMS_MAX_ENTRIES_RESOLVED: OnceLock<usize> = OnceLock::new();
static LASM_DB_OP_SEQUENCE_MAX_RESOLVED: OnceLock<usize> = OnceLock::new();
static LASM_DB_QUERY_ONE_ROW_MAX_BYTES_RESOLVED: OnceLock<usize> = OnceLock::new();
static LASM_DB_QUERY_ONE_ROW_MAX_COLUMNS_RESOLVED: OnceLock<usize> = OnceLock::new();
static LASM_DB_SQL_TEMPLATE_MAX_BYTES_OVERRIDE: AtomicUsize = AtomicUsize::new(0);
static LASM_DB_PARAMS_MAX_BYTES_OVERRIDE: AtomicUsize = AtomicUsize::new(0);
static LASM_DB_PARAMS_MAX_ENTRIES_OVERRIDE: AtomicUsize = AtomicUsize::new(0);
static LASM_DB_QUERY_ONE_ROW_MAX_BYTES_OVERRIDE: AtomicUsize = AtomicUsize::new(0);
static LASM_DB_QUERY_ONE_ROW_MAX_COLUMNS_OVERRIDE: AtomicUsize = AtomicUsize::new(0);

#[inline(always)]
fn resolve_lasm_db_sql_template_max_bytes() -> usize {
    let override_value = LASM_DB_SQL_TEMPLATE_MAX_BYTES_OVERRIDE.load(Ordering::Relaxed);
    if override_value != 0 {
        return override_value.clamp(
            LASM_DB_SQL_TEMPLATE_MAX_BYTES_MIN,
            LASM_DB_SQL_TEMPLATE_MAX_BYTES_MAX,
        );
    }
    *LASM_DB_SQL_TEMPLATE_MAX_BYTES_RESOLVED.get_or_init(|| {
        let Ok(raw) = env::var(LASM_DB_SQL_TEMPLATE_MAX_BYTES_ENV) else {
            return LASM_DB_SQL_TEMPLATE_MAX_BYTES_DEFAULT;
        };
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return LASM_DB_SQL_TEMPLATE_MAX_BYTES_DEFAULT;
        }
        let Ok(parsed) = trimmed.parse::<usize>() else {
            return LASM_DB_SQL_TEMPLATE_MAX_BYTES_DEFAULT;
        };
        parsed.clamp(
            LASM_DB_SQL_TEMPLATE_MAX_BYTES_MIN,
            LASM_DB_SQL_TEMPLATE_MAX_BYTES_MAX,
        )
    })
}

pub(crate) fn lasm_db_sql_template_max_bytes_limit() -> usize {
    resolve_lasm_db_sql_template_max_bytes()
}

pub(crate) fn set_lasm_db_sql_template_max_bytes_override(value: Option<usize>) {
    LASM_DB_SQL_TEMPLATE_MAX_BYTES_OVERRIDE.store(value.unwrap_or(0), Ordering::Relaxed);
}

#[inline(always)]
fn enforce_lasm_db_sql_template_max_bytes(
    response: &mut sec4_core::HttpResponse,
    operation: &str,
    template: &str,
    trace_id: &str,
) -> bool {
    let max_bytes = resolve_lasm_db_sql_template_max_bytes();
    let template_bytes = template.as_bytes().len();
    if template_bytes <= max_bytes {
        return true;
    }
    let code = match operation {
        "exec" => "DB.EXEC_INVALID",
        "execTx" => "DB.EXEC_TX_INVALID",
        "queryOne" => "DB.QUERY_ONE_INVALID",
        _ => "DB.SQL_TEMPLATE_INVALID",
    };
    let message = format!("sql.q query template exceeds configured max bytes ({max_bytes})");
    set_lasm_json_response(
        response,
        400,
        &lasm_error_envelope(code, "validation", message.as_str(), 400, trace_id),
    );
    false
}

#[inline(always)]
fn resolve_lasm_db_params_max_bytes() -> usize {
    let override_value = LASM_DB_PARAMS_MAX_BYTES_OVERRIDE.load(Ordering::Relaxed);
    if override_value != 0 {
        return override_value.clamp(LASM_DB_PARAMS_MAX_BYTES_MIN, LASM_DB_PARAMS_MAX_BYTES_MAX);
    }
    *LASM_DB_PARAMS_MAX_BYTES_RESOLVED.get_or_init(|| {
        let Ok(raw) = env::var(LASM_DB_PARAMS_MAX_BYTES_ENV) else {
            return LASM_DB_PARAMS_MAX_BYTES_DEFAULT;
        };
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return LASM_DB_PARAMS_MAX_BYTES_DEFAULT;
        }
        let Ok(parsed) = trimmed.parse::<usize>() else {
            return LASM_DB_PARAMS_MAX_BYTES_DEFAULT;
        };
        parsed.clamp(LASM_DB_PARAMS_MAX_BYTES_MIN, LASM_DB_PARAMS_MAX_BYTES_MAX)
    })
}

pub(crate) fn lasm_db_params_max_bytes_limit() -> usize {
    resolve_lasm_db_params_max_bytes()
}

pub(crate) fn set_lasm_db_params_max_bytes_override(value: Option<usize>) {
    LASM_DB_PARAMS_MAX_BYTES_OVERRIDE.store(value.unwrap_or(0), Ordering::Relaxed);
}

#[inline(always)]
fn enforce_lasm_db_params_max_bytes(
    response: &mut sec4_core::HttpResponse,
    operation: &str,
    params: &str,
    trace_id: &str,
) -> bool {
    let max_bytes = resolve_lasm_db_params_max_bytes();
    let params_bytes = params.as_bytes().len();
    if params_bytes <= max_bytes {
        return true;
    }
    let code = match operation {
        "exec" => "DB.EXEC_INVALID",
        "execTx" => "DB.EXEC_TX_INVALID",
        "queryOne" => "DB.QUERY_ONE_INVALID",
        _ => "DB.OPERATION_INVALID",
    };
    let message = format!("sql.q params payload exceeds configured max bytes ({max_bytes})");
    set_lasm_json_response(
        response,
        400,
        &lasm_error_envelope(code, "validation", message.as_str(), 400, trace_id),
    );
    false
}

#[inline(always)]
fn enforce_lasm_db_params_required(
    response: &mut sec4_core::HttpResponse,
    operation: &str,
    params: &str,
    trace_id: &str,
) -> bool {
    if !params.trim().is_empty() {
        return true;
    }
    let code = match operation {
        "exec" => "DB.EXEC_INVALID",
        "execTx" => "DB.EXEC_TX_INVALID",
        "queryOne" => "DB.QUERY_ONE_INVALID",
        _ => "DB.OPERATION_INVALID",
    };
    set_lasm_json_response(
        response,
        400,
        &lasm_error_envelope(
            code,
            "validation",
            "sql.q params payload is required",
            400,
            trace_id,
        ),
    );
    false
}

#[inline(always)]
fn resolve_lasm_db_params_max_entries() -> usize {
    let override_value = LASM_DB_PARAMS_MAX_ENTRIES_OVERRIDE.load(Ordering::Relaxed);
    if override_value != 0 {
        return override_value.clamp(
            LASM_DB_PARAMS_MAX_ENTRIES_MIN,
            LASM_DB_PARAMS_MAX_ENTRIES_MAX,
        );
    }
    *LASM_DB_PARAMS_MAX_ENTRIES_RESOLVED.get_or_init(|| {
        let Ok(raw) = env::var(LASM_DB_PARAMS_MAX_ENTRIES_ENV) else {
            return LASM_DB_PARAMS_MAX_ENTRIES_DEFAULT;
        };
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return LASM_DB_PARAMS_MAX_ENTRIES_DEFAULT;
        }
        let Ok(parsed) = trimmed.parse::<usize>() else {
            return LASM_DB_PARAMS_MAX_ENTRIES_DEFAULT;
        };
        parsed.clamp(
            LASM_DB_PARAMS_MAX_ENTRIES_MIN,
            LASM_DB_PARAMS_MAX_ENTRIES_MAX,
        )
    })
}

pub(crate) fn lasm_db_params_max_entries_limit() -> usize {
    resolve_lasm_db_params_max_entries()
}

pub(crate) fn set_lasm_db_params_max_entries_override(value: Option<usize>) {
    LASM_DB_PARAMS_MAX_ENTRIES_OVERRIDE.store(value.unwrap_or(0), Ordering::Relaxed);
}

#[inline(always)]
fn lasm_db_params_entry_count(
    parsed: Option<&serde_json::Value>,
    normalized_params: &str,
) -> usize {
    match parsed {
        Some(serde_json::Value::Array(values)) => values.len(),
        Some(serde_json::Value::Object(values)) => values.len(),
        Some(serde_json::Value::Null) => 0,
        Some(_) => 1,
        None => {
            if normalized_params == "0" {
                0
            } else {
                1
            }
        }
    }
}

#[inline(always)]
fn resolve_lasm_db_op_sequence_max() -> usize {
    *LASM_DB_OP_SEQUENCE_MAX_RESOLVED.get_or_init(|| {
        std::env::var(LASM_DB_OP_SEQUENCE_MAX_ENV)
            .ok()
            .and_then(|raw| raw.trim().parse::<usize>().ok())
            .map(|value| value.clamp(LASM_DB_OP_SEQUENCE_MAX_MIN, LASM_DB_OP_SEQUENCE_MAX_MAX))
            .unwrap_or(LASM_DB_OP_SEQUENCE_MAX_DEFAULT)
    })
}

#[inline(always)]
pub(crate) fn lasm_db_op_sequence_max_limit() -> usize {
    resolve_lasm_db_op_sequence_max()
}

#[inline(always)]
fn enforce_lasm_db_params_max_entries(
    response: &mut sec4_core::HttpResponse,
    operation: &str,
    parsed: Option<&serde_json::Value>,
    normalized_params: &str,
    trace_id: &str,
) -> bool {
    let max_entries = resolve_lasm_db_params_max_entries();
    let entries = lasm_db_params_entry_count(parsed, normalized_params);
    if entries <= max_entries {
        return true;
    }
    let code = match operation {
        "exec" => "DB.EXEC_INVALID",
        "execTx" => "DB.EXEC_TX_INVALID",
        "queryOne" => "DB.QUERY_ONE_INVALID",
        _ => "DB.OPERATION_INVALID",
    };
    let message =
        format!("sql.q params entry count exceeds configured max entries ({max_entries})");
    set_lasm_json_response(
        response,
        400,
        &lasm_error_envelope(code, "validation", message.as_str(), 400, trace_id),
    );
    false
}

#[inline(always)]
fn resolve_lasm_db_query_one_row_max_bytes() -> usize {
    let override_value = LASM_DB_QUERY_ONE_ROW_MAX_BYTES_OVERRIDE.load(Ordering::Relaxed);
    if override_value != 0 {
        return override_value.clamp(
            LASM_DB_QUERY_ONE_ROW_MAX_BYTES_MIN,
            LASM_DB_QUERY_ONE_ROW_MAX_BYTES_MAX,
        );
    }
    *LASM_DB_QUERY_ONE_ROW_MAX_BYTES_RESOLVED.get_or_init(|| {
        let Ok(raw) = env::var(LASM_DB_QUERY_ONE_ROW_MAX_BYTES_ENV) else {
            return LASM_DB_QUERY_ONE_ROW_MAX_BYTES_DEFAULT;
        };
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return LASM_DB_QUERY_ONE_ROW_MAX_BYTES_DEFAULT;
        }
        let Ok(parsed) = trimmed.parse::<usize>() else {
            return LASM_DB_QUERY_ONE_ROW_MAX_BYTES_DEFAULT;
        };
        parsed.clamp(
            LASM_DB_QUERY_ONE_ROW_MAX_BYTES_MIN,
            LASM_DB_QUERY_ONE_ROW_MAX_BYTES_MAX,
        )
    })
}

pub(crate) fn lasm_db_query_one_row_max_bytes_limit() -> usize {
    resolve_lasm_db_query_one_row_max_bytes()
}

pub(crate) fn set_lasm_db_query_one_row_max_bytes_override(value: Option<usize>) {
    LASM_DB_QUERY_ONE_ROW_MAX_BYTES_OVERRIDE.store(value.unwrap_or(0), Ordering::Relaxed);
}

#[inline(always)]
fn enforce_lasm_db_query_one_row_max_bytes(
    response: &mut sec4_core::HttpResponse,
    row: &str,
    trace_id: &str,
) -> bool {
    let max_bytes = resolve_lasm_db_query_one_row_max_bytes();
    if row.as_bytes().len() <= max_bytes {
        return true;
    }
    let message = format!("db.queryOne row payload exceeds configured max bytes ({max_bytes})");
    set_lasm_json_response(
        response,
        413,
        &lasm_error_envelope(
            "DB.QUERY_ONE_ROW_LIMIT",
            "resource_limit",
            message.as_str(),
            413,
            trace_id,
        ),
    );
    false
}

#[inline(always)]
fn resolve_lasm_db_query_one_row_max_columns() -> usize {
    let override_value = LASM_DB_QUERY_ONE_ROW_MAX_COLUMNS_OVERRIDE.load(Ordering::Relaxed);
    if override_value != 0 {
        return override_value.clamp(
            LASM_DB_QUERY_ONE_ROW_MAX_COLUMNS_MIN,
            LASM_DB_QUERY_ONE_ROW_MAX_COLUMNS_MAX,
        );
    }
    *LASM_DB_QUERY_ONE_ROW_MAX_COLUMNS_RESOLVED.get_or_init(|| {
        let Ok(raw) = env::var(LASM_DB_QUERY_ONE_ROW_MAX_COLUMNS_ENV) else {
            return LASM_DB_QUERY_ONE_ROW_MAX_COLUMNS_DEFAULT;
        };
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return LASM_DB_QUERY_ONE_ROW_MAX_COLUMNS_DEFAULT;
        }
        let Ok(parsed) = trimmed.parse::<usize>() else {
            return LASM_DB_QUERY_ONE_ROW_MAX_COLUMNS_DEFAULT;
        };
        parsed.clamp(
            LASM_DB_QUERY_ONE_ROW_MAX_COLUMNS_MIN,
            LASM_DB_QUERY_ONE_ROW_MAX_COLUMNS_MAX,
        )
    })
}

pub(crate) fn lasm_db_query_one_row_max_columns_limit() -> usize {
    resolve_lasm_db_query_one_row_max_columns()
}

pub(crate) fn set_lasm_db_query_one_row_max_columns_override(value: Option<usize>) {
    LASM_DB_QUERY_ONE_ROW_MAX_COLUMNS_OVERRIDE.store(value.unwrap_or(0), Ordering::Relaxed);
}

#[inline(always)]
fn lasm_db_query_one_row_column_count(row_object: &serde_json::Value) -> usize {
    match row_object {
        serde_json::Value::Object(values) => values.len(),
        serde_json::Value::Null => 0,
        _ => 1,
    }
}

#[inline(always)]
fn enforce_lasm_db_query_one_row_max_columns(
    response: &mut sec4_core::HttpResponse,
    row_object: &serde_json::Value,
    trace_id: &str,
) -> bool {
    let max_columns = resolve_lasm_db_query_one_row_max_columns();
    let columns = lasm_db_query_one_row_column_count(row_object);
    if columns <= max_columns {
        return true;
    }
    let message =
        format!("db.queryOne row column count exceeds configured max columns ({max_columns})");
    set_lasm_json_response(
        response,
        413,
        &lasm_error_envelope(
            "DB.QUERY_ONE_COLUMN_LIMIT",
            "resource_limit",
            message.as_str(),
            413,
            trace_id,
        ),
    );
    false
}

fn persist_lasm_db_record_with_capacity_guard(
    state: &mut LasmDynamicResponseState,
    record: &LasmDbRecord,
) {
    let dropped_before = state.db_records_dropped_total;
    let history_overflowed = append_lasm_dynamic_db_record(state, record.clone());
    if let Err(message) = persist_lasm_dynamic_db_record_append(state, record) {
        eprintln!("warning: LASM dynamic records store persistence failed: {message}");
    }
    // Avoid O(n) full-store sync on every request once history is over capacity.
    // We still perform periodic compaction sync to keep persistent history bounded.
    if history_overflowed
        && state.db_records_dropped_total != dropped_before
        && state
            .db_records_dropped_total
            .is_multiple_of(LASM_DB_RECORDS_COMPACTION_SYNC_DROPS_INTERVAL)
    {
        if let Err(message) = persist_lasm_dynamic_db_records_full_sync(state) {
            eprintln!(
                "warning: LASM dynamic records store compaction sync failed after \
                 in-memory overflow: {message}"
            );
        }
    }
}

fn build_lasm_postgres_thread_local_config(
    state: &LasmDynamicResponseState,
) -> Result<LasmPostgresThreadLocalConfig, String> {
    let dsn = state.db_records_postgres_dsn.as_deref().ok_or_else(|| {
        "db adapter postgres requires SEC4_RT_LASM_DB_POSTGRES_DSN to be set".to_string()
    })?;
    Ok(LasmPostgresThreadLocalConfig {
        dsn: dsn.to_string(),
        tls_mode: state.db_postgres_tls_mode,
        statement_timeout_ms: state.db_postgres_statement_timeout_ms.max(1),
        lock_timeout_ms: state.db_postgres_lock_timeout_ms.max(1),
        connect_timeout_ms: state.db_postgres_connect_timeout_ms.max(1),
        retryable_conflict_retry_max: state.db_postgres_retryable_conflict_retry_max,
    })
}

fn append_lasm_db_record_in_memory_with_compaction_snapshot(
    state: &mut LasmDynamicResponseState,
    record: LasmDbRecord,
) -> (LasmDbRecord, Option<Vec<LasmDbRecord>>) {
    let dropped_before = state.db_records_dropped_total;
    let history_overflowed = append_lasm_dynamic_db_record(state, record.clone());
    let should_full_sync = history_overflowed
        && state.db_records_dropped_total != dropped_before
        && state
            .db_records_dropped_total
            .is_multiple_of(LASM_DB_RECORDS_COMPACTION_SYNC_DROPS_INTERVAL);
    let compaction_snapshot = if should_full_sync {
        Some(state.db_records.clone())
    } else {
        None
    };
    (record, compaction_snapshot)
}

struct PreparedLasmDbOperationParams {
    postgres: Option<(String, Vec<LasmPostgresParam>)>,
    sqlite: Option<LasmSqliteQueryParams>,
}

enum LasmExecTxSource {
    AllocateFromDb(i64),
    ExistingTx(i64),
}

fn prepare_lasm_db_operation_params(
    db_records_adapter: LasmDbRecordsAdapter,
    validation_code: &'static str,
    template: &str,
    params: &str,
    parsed_params: Option<&serde_json::Value>,
    response: &mut sec4_core::HttpResponse,
    trace_id: &str,
) -> Option<PreparedLasmDbOperationParams> {
    let postgres_preparsed = if db_records_adapter == LasmDbRecordsAdapter::Postgres {
        Some(match parsed_params {
            Some(parsed) => parse_lasm_postgres_query_template_and_params_value(template, parsed),
            None => parse_lasm_postgres_query_template_and_params(template, params),
        })
    } else {
        None
    };
    let sqlite_params = if db_records_adapter == LasmDbRecordsAdapter::Sqlite {
        Some(match parsed_params {
            Some(parsed) => parse_lasm_sqlite_query_params_value(parsed),
            None => parse_lasm_sqlite_query_params(params),
        })
    } else {
        None
    };
    if let Some(Err(message)) = sqlite_params {
        set_lasm_json_response(
            response,
            400,
            &lasm_error_envelope(
                validation_code,
                "validation",
                message.as_str(),
                400,
                trace_id,
            ),
        );
        return None;
    }
    if let Some(Err(message)) = postgres_preparsed {
        set_lasm_json_response(
            response,
            400,
            &lasm_error_envelope(
                validation_code,
                "validation",
                message.as_str(),
                400,
                trace_id,
            ),
        );
        return None;
    }
    Some(PreparedLasmDbOperationParams {
        postgres: postgres_preparsed.and_then(Result::ok),
        sqlite: sqlite_params.and_then(Result::ok),
    })
}

fn set_lasm_dynamic_state_unavailable_response(
    response: &mut sec4_core::HttpResponse,
    trace_id: &str,
) {
    set_lasm_json_response(
        response,
        500,
        &lasm_error_envelope(
            "HTTP.INTERNAL",
            "internal",
            "dynamic response state unavailable",
            500,
            trace_id,
        ),
    );
}

fn lock_lasm_dynamic_state_or_respond<'a>(
    dynamic_state: &'a Mutex<LasmDynamicResponseState>,
    response: &mut sec4_core::HttpResponse,
    trace_id: &str,
) -> Option<MutexGuard<'a, LasmDynamicResponseState>> {
    match dynamic_state.lock() {
        Ok(state) => Some(state),
        Err(_) => {
            set_lasm_dynamic_state_unavailable_response(response, trace_id);
            None
        }
    }
}

fn set_lasm_db_runtime_error_response(
    response: &mut sec4_core::HttpResponse,
    operation: &str,
    message: &str,
    trace_id: &str,
) {
    let (status, code, kind) = classify_lasm_db_runtime_error(operation, message);
    set_lasm_json_response(
        response,
        status,
        &lasm_error_envelope(code, kind, message, status, trace_id),
    );
}

fn resolve_lasm_exec_tx_source(
    tx_db_source: Option<&str>,
    tx_handle_raw: Option<&str>,
    response: &mut sec4_core::HttpResponse,
    trace_id: &str,
) -> Option<LasmExecTxSource> {
    if tx_db_source.is_some() && tx_handle_raw.is_some() {
        set_lasm_json_response(
            response,
            400,
            &lasm_error_envelope(
                "DB.EXEC_TX_INVALID",
                "validation",
                "db.execTx must include either tx handle or db.tx(dbCap) source, not both",
                400,
                trace_id,
            ),
        );
        return None;
    }
    let tx_source = if let Some(db_raw) = tx_db_source {
        let Some(db_value) = parse_lasm_positive_i64(db_raw.trim()) else {
            set_lasm_json_response(
                response,
                400,
                &lasm_error_envelope(
                    "DB.EXEC_TX_INVALID",
                    "validation",
                    "db.execTx requires transaction and query handles",
                    400,
                    trace_id,
                ),
            );
            return None;
        };
        LasmExecTxSource::AllocateFromDb(db_value)
    } else {
        let Some(tx_raw) = tx_handle_raw else {
            set_lasm_json_response(
                response,
                400,
                &lasm_error_envelope(
                    "DB.EXEC_TX_INVALID",
                    "validation",
                    "db.execTx requires transaction and query handles",
                    400,
                    trace_id,
                ),
            );
            return None;
        };
        let Some(tx_value) = parse_lasm_positive_i64(tx_raw.trim()) else {
            set_lasm_json_response(
                response,
                400,
                &lasm_error_envelope(
                    "DB.EXEC_TX_INVALID",
                    "validation",
                    "db.execTx requires transaction and query handles",
                    400,
                    trace_id,
                ),
            );
            return None;
        };
        LasmExecTxSource::ExistingTx(tx_value)
    };
    if let LasmExecTxSource::AllocateFromDb(db_value) = &tx_source {
        if !is_lasm_valid_db_cap_handle(*db_value) {
            set_lasm_json_response(
                response,
                400,
                &lasm_error_envelope(
                    "DB.EXEC_TX_INVALID",
                    "validation",
                    "db.execTx requires db.tx(dbCap) with valid db capability handle",
                    400,
                    trace_id,
                ),
            );
            return None;
        }
    }
    Some(tx_source)
}

fn resolve_lasm_exec_tx_state_bindings(
    state: &mut LasmDynamicResponseState,
    tx_source: &LasmExecTxSource,
    response: &mut sec4_core::HttpResponse,
    trace_id: &str,
) -> Option<(i64, i64, Option<i64>)> {
    match tx_source {
        LasmExecTxSource::AllocateFromDb(db_value) => {
            let Some(tx_value) = allocate_lasm_db_tx_handle(state, *db_value) else {
                set_lasm_json_response(
                    response,
                    500,
                    &lasm_error_envelope(
                        "DB.TX_INTERNAL",
                        "internal",
                        "db.tx runtime failure",
                        500,
                        trace_id,
                    ),
                );
                return None;
            };
            Some((*db_value, tx_value, Some(tx_value)))
        }
        LasmExecTxSource::ExistingTx(tx_value) => {
            let Some(db_value) = state.db_tx_handles.get(tx_value).copied() else {
                set_lasm_json_response(
                    response,
                    400,
                    &lasm_error_envelope(
                        "DB.EXEC_TX_HANDLE_INVALID",
                        "validation",
                        "db.execTx transaction handle must come from db.tx",
                        400,
                        trace_id,
                    ),
                );
                return None;
            };
            Some((db_value, *tx_value, None))
        }
    }
}

fn cleanup_lasm_exec_tx_allocated_handle(
    dynamic_state: &Mutex<LasmDynamicResponseState>,
    tx_handle: Option<i64>,
) {
    if let Some(tx_handle) = tx_handle {
        if let Ok(mut state) = dynamic_state.lock() {
            state.db_tx_handles.remove(&tx_handle);
        }
    }
}

fn is_lasm_internal_db_indexed_header_name(header_name: &str, base_name: &str) -> bool {
    if header_name.len() <= base_name.len() + 1 {
        return false;
    }
    if !header_name[..base_name.len()].eq_ignore_ascii_case(base_name) {
        return false;
    }
    if header_name.as_bytes()[base_name.len()] != b'-' {
        return false;
    }
    let suffix = &header_name[(base_name.len() + 1)..];
    !suffix.is_empty() && suffix.chars().all(|ch| ch.is_ascii_digit())
}

fn response_has_lasm_internal_db_indexed_headers(response: &sec4_core::HttpResponse) -> bool {
    const INDEXED_BASE_HEADERS: [&str; 7] = [
        LASM_INTERNAL_DB_OP_HEADER,
        LASM_INTERNAL_DB_HANDLE_HEADER,
        LASM_INTERNAL_DB_TEMPLATE_HEADER,
        LASM_INTERNAL_DB_PARAMS_HEADER,
        LASM_INTERNAL_DB_TX_HEADER,
        LASM_INTERNAL_DB_TX_DB_HEADER,
        LASM_INTERNAL_DB_ROW_SCHEMA_HEADER,
    ];
    for header_name in response.headers.keys() {
        for base_name in INDEXED_BASE_HEADERS {
            if is_lasm_internal_db_indexed_header_name(header_name.as_str(), base_name) {
                return true;
            }
        }
    }
    false
}

pub(crate) fn apply_lasm_internal_db_operation_materialization(
    response: &mut sec4_core::HttpResponse,
    request: &LasmRunRequest,
    path_params: &BTreeMap<String, String>,
    dynamic_state: &Mutex<LasmDynamicResponseState>,
    db_records_adapter: LasmDbRecordsAdapter,
    trace_id: &str,
) -> bool {
    let operation_count = if let Some(raw_operation_count) =
        take_lasm_internal_header_value(response, LASM_INTERNAL_DB_OP_COUNT_HEADER)
    {
        let trimmed = raw_operation_count.trim();
        let Some(parsed) = trimmed.parse::<usize>().ok() else {
            set_lasm_json_response(
                response,
                400,
                &lasm_error_envelope(
                    "DB.OPERATION_INVALID",
                    "validation",
                    "invalid internal db operation sequence marker",
                    400,
                    trace_id,
                ),
            );
            return true;
        };
        if parsed < 2 {
            set_lasm_json_response(
                response,
                400,
                &lasm_error_envelope(
                    "DB.OPERATION_INVALID",
                    "validation",
                    "internal db operation sequence marker value must be >= 2",
                    400,
                    trace_id,
                ),
            );
            return true;
        }
        parsed
    } else {
        0
    };
    if operation_count == 0 && response_has_lasm_internal_db_indexed_headers(response) {
        set_lasm_json_response(
            response,
            400,
            &lasm_error_envelope(
                "DB.OPERATION_INVALID",
                "validation",
                "indexed internal db operation markers require operation count marker",
                400,
                trace_id,
            ),
        );
        return true;
    }
    let operation_sequence_max = resolve_lasm_db_op_sequence_max();
    if operation_count > operation_sequence_max {
        let message = format!(
            "db operation sequence exceeds maximum supported operations per handler ({operation_sequence_max})"
        );
        set_lasm_json_response(
            response,
            400,
            &lasm_error_envelope(
                "DB.OPERATION_INVALID",
                "validation",
                message.as_str(),
                400,
                trace_id,
            ),
        );
        return true;
    }
    if operation_count > 1 {
        let mut sequence_tx_handles_by_source = BTreeMap::<i64, i64>::new();
        for index in 0..operation_count {
            let Some(raw_operation) = take_lasm_internal_header_value_indexed(
                response,
                LASM_INTERNAL_DB_OP_HEADER,
                index,
            ) else {
                set_lasm_json_response(
                    response,
                    400,
                    &lasm_error_envelope(
                        "DB.OPERATION_INVALID",
                        "validation",
                        "missing internal db operation marker for operation sequence",
                        400,
                        trace_id,
                    ),
                );
                return true;
            };
            let operation = raw_operation.trim().to_string();

            response.headers.remove(LASM_INTERNAL_DB_OP_HEADER);
            response.headers.remove(LASM_INTERNAL_DB_HANDLE_HEADER);
            response.headers.remove(LASM_INTERNAL_DB_TEMPLATE_HEADER);
            response.headers.remove(LASM_INTERNAL_DB_PARAMS_HEADER);
            response.headers.remove(LASM_INTERNAL_DB_TX_HEADER);
            response.headers.remove(LASM_INTERNAL_DB_TX_DB_HEADER);
            response.headers.remove(LASM_INTERNAL_DB_TX_RESULT_HEADER);
            response
                .headers
                .remove(LASM_INTERNAL_DB_TX_SEQUENCE_RETAIN_HEADER);
            response.headers.remove(LASM_INTERNAL_DB_ROW_SCHEMA_HEADER);
            response
                .headers
                .insert(LASM_INTERNAL_DB_OP_HEADER.to_string(), raw_operation);
            if let Some(value) = take_lasm_internal_header_value_indexed(
                response,
                LASM_INTERNAL_DB_HANDLE_HEADER,
                index,
            ) {
                response
                    .headers
                    .insert(LASM_INTERNAL_DB_HANDLE_HEADER.to_string(), value);
            }
            if let Some(value) = take_lasm_internal_header_value_indexed(
                response,
                LASM_INTERNAL_DB_TEMPLATE_HEADER,
                index,
            ) {
                response
                    .headers
                    .insert(LASM_INTERNAL_DB_TEMPLATE_HEADER.to_string(), value);
            }
            if let Some(value) = take_lasm_internal_header_value_indexed(
                response,
                LASM_INTERNAL_DB_PARAMS_HEADER,
                index,
            ) {
                response
                    .headers
                    .insert(LASM_INTERNAL_DB_PARAMS_HEADER.to_string(), value);
            }
            let mut sequence_allocated_tx_source: Option<i64> = None;
            if operation == "execTx" {
                let raw_tx_db = take_lasm_internal_header_value_indexed(
                    response,
                    LASM_INTERNAL_DB_TX_DB_HEADER,
                    index,
                );
                let raw_tx_handle = take_lasm_internal_header_value_indexed(
                    response,
                    LASM_INTERNAL_DB_TX_HEADER,
                    index,
                );
                if raw_tx_db.is_some() && raw_tx_handle.is_some() {
                    set_lasm_json_response(
                        response,
                        400,
                        &lasm_error_envelope(
                            "DB.EXEC_TX_INVALID",
                            "validation",
                            "db.execTx must include either tx handle or db.tx(dbCap) source, not both",
                            400,
                            trace_id,
                        ),
                    );
                    cleanup_lasm_internal_db_sequence_tx_handles(
                        dynamic_state,
                        sequence_tx_handles_by_source.values().copied(),
                    );
                    return true;
                }
                if let Some(raw_tx_db) = raw_tx_db {
                    let tx_db_source_raw = materialize_lasm_internal_header_value(
                        raw_tx_db.clone(),
                        request,
                        path_params,
                    );
                    let Some(tx_db_source) = parse_lasm_positive_i64(tx_db_source_raw.trim())
                    else {
                        set_lasm_json_response(
                            response,
                            400,
                            &lasm_error_envelope(
                                "DB.EXEC_TX_INVALID",
                                "validation",
                                "db.execTx requires transaction and query handles",
                                400,
                                trace_id,
                            ),
                        );
                        cleanup_lasm_internal_db_sequence_tx_handles(
                            dynamic_state,
                            sequence_tx_handles_by_source.values().copied(),
                        );
                        return true;
                    };
                    if !is_lasm_valid_db_cap_handle(tx_db_source) {
                        set_lasm_json_response(
                            response,
                            400,
                            &lasm_error_envelope(
                                "DB.EXEC_TX_INVALID",
                                "validation",
                                "db.execTx requires db.tx(dbCap) with valid db capability handle",
                                400,
                                trace_id,
                            ),
                        );
                        cleanup_lasm_internal_db_sequence_tx_handles(
                            dynamic_state,
                            sequence_tx_handles_by_source.values().copied(),
                        );
                        return true;
                    }
                    if let Some(existing_tx_handle) =
                        sequence_tx_handles_by_source.get(&tx_db_source).copied()
                    {
                        response.headers.insert(
                            LASM_INTERNAL_DB_TX_HEADER.to_string(),
                            existing_tx_handle.to_string(),
                        );
                    } else {
                        response
                            .headers
                            .insert(LASM_INTERNAL_DB_TX_DB_HEADER.to_string(), raw_tx_db);
                        response.headers.insert(
                            LASM_INTERNAL_DB_TX_SEQUENCE_RETAIN_HEADER.to_string(),
                            "1".to_string(),
                        );
                        sequence_allocated_tx_source = Some(tx_db_source);
                    }
                } else if let Some(raw_tx_handle) = raw_tx_handle {
                    response
                        .headers
                        .insert(LASM_INTERNAL_DB_TX_HEADER.to_string(), raw_tx_handle);
                }
            } else {
                if let Some(value) = take_lasm_internal_header_value_indexed(
                    response,
                    LASM_INTERNAL_DB_TX_HEADER,
                    index,
                ) {
                    response
                        .headers
                        .insert(LASM_INTERNAL_DB_TX_HEADER.to_string(), value);
                }
                if let Some(value) = take_lasm_internal_header_value_indexed(
                    response,
                    LASM_INTERNAL_DB_TX_DB_HEADER,
                    index,
                ) {
                    response
                        .headers
                        .insert(LASM_INTERNAL_DB_TX_DB_HEADER.to_string(), value);
                }
            }
            if let Some(value) = take_lasm_internal_header_value_indexed(
                response,
                LASM_INTERNAL_DB_ROW_SCHEMA_HEADER,
                index,
            ) {
                response
                    .headers
                    .insert(LASM_INTERNAL_DB_ROW_SCHEMA_HEADER.to_string(), value);
            }

            if !apply_lasm_internal_db_operation_materialization_single(
                response,
                request,
                path_params,
                dynamic_state,
                db_records_adapter,
                trace_id,
            ) {
                set_lasm_json_response(
                    response,
                    400,
                    &lasm_error_envelope(
                        "DB.OPERATION_INVALID",
                        "validation",
                        "missing internal db operation marker",
                        400,
                        trace_id,
                    ),
                );
                cleanup_lasm_internal_db_sequence_tx_handles(
                    dynamic_state,
                    sequence_tx_handles_by_source.values().copied(),
                );
                return true;
            }
            if response.status >= 400 {
                cleanup_lasm_internal_db_sequence_tx_handles(
                    dynamic_state,
                    sequence_tx_handles_by_source.values().copied(),
                );
                return true;
            }
            if let Some(tx_db_source) = sequence_allocated_tx_source {
                let Some(tx_handle_raw) =
                    take_lasm_internal_header_value(response, LASM_INTERNAL_DB_TX_RESULT_HEADER)
                else {
                    set_lasm_json_response(
                        response,
                        500,
                        &lasm_error_envelope(
                            "DB.TX_INTERNAL",
                            "internal",
                            "db.execTx runtime did not publish transaction handle marker",
                            500,
                            trace_id,
                        ),
                    );
                    cleanup_lasm_internal_db_sequence_tx_handles(
                        dynamic_state,
                        sequence_tx_handles_by_source.values().copied(),
                    );
                    return true;
                };
                let Some(tx_handle) = parse_lasm_positive_i64(tx_handle_raw.as_str()) else {
                    set_lasm_json_response(
                        response,
                        500,
                        &lasm_error_envelope(
                            "DB.TX_INTERNAL",
                            "internal",
                            "db.tx runtime failure",
                            500,
                            trace_id,
                        ),
                    );
                    cleanup_lasm_internal_db_sequence_tx_handles(
                        dynamic_state,
                        sequence_tx_handles_by_source.values().copied(),
                    );
                    return true;
                };
                sequence_tx_handles_by_source.insert(tx_db_source, tx_handle);
            }
        }
        cleanup_lasm_internal_db_sequence_tx_handles(
            dynamic_state,
            sequence_tx_handles_by_source.values().copied(),
        );
        return true;
    }

    apply_lasm_internal_db_operation_materialization_single(
        response,
        request,
        path_params,
        dynamic_state,
        db_records_adapter,
        trace_id,
    )
}

fn apply_lasm_internal_db_operation_materialization_single(
    response: &mut sec4_core::HttpResponse,
    request: &LasmRunRequest,
    path_params: &BTreeMap<String, String>,
    dynamic_state: &Mutex<LasmDynamicResponseState>,
    db_records_adapter: LasmDbRecordsAdapter,
    trace_id: &str,
) -> bool {
    let Some(raw_operation) = take_lasm_internal_header_value(response, LASM_INTERNAL_DB_OP_HEADER)
    else {
        return false;
    };
    let operation = materialize_lasm_internal_header_value(raw_operation, request, path_params);
    let operation = operation.trim();
    match operation {
        "listRecords" => {
            apply_lasm_db_list_records_response_materialization(
                response,
                request,
                dynamic_state,
                trace_id,
            );
            true
        }
        "exec" => {
            let Some(raw_template_header) =
                take_lasm_internal_header_value(response, LASM_INTERNAL_DB_TEMPLATE_HEADER)
            else {
                set_lasm_json_response(
                    response,
                    400,
                    &lasm_error_envelope(
                        "DB.EXEC_INVALID",
                        "validation",
                        "db.exec requires db capability and query handle",
                        400,
                        trace_id,
                    ),
                );
                return true;
            };
            let template =
                materialize_lasm_internal_header_value(raw_template_header, request, path_params);
            if !enforce_lasm_db_sql_template_max_bytes(
                response,
                "exec",
                template.as_str(),
                trace_id,
            ) {
                return true;
            }
            if template.trim().is_empty() {
                set_lasm_json_response(
                    response,
                    400,
                    &lasm_error_envelope(
                        "DB.SQL_TEMPLATE_INVALID",
                        "validation",
                        "sql.q query template is required",
                        400,
                        trace_id,
                    ),
                );
                return true;
            }
            let Some(raw_params_header) =
                take_lasm_internal_header_value(response, LASM_INTERNAL_DB_PARAMS_HEADER)
            else {
                set_lasm_json_response(
                    response,
                    400,
                    &lasm_error_envelope(
                        "DB.EXEC_INVALID",
                        "validation",
                        "db.exec requires db capability and query handle",
                        400,
                        trace_id,
                    ),
                );
                return true;
            };
            let params =
                materialize_lasm_internal_header_value(raw_params_header, request, path_params);
            if !enforce_lasm_db_params_required(response, "exec", params.as_str(), trace_id) {
                return true;
            }
            if !enforce_lasm_db_params_max_bytes(response, "exec", params.as_str(), trace_id) {
                return true;
            }
            let Some(raw_db_header) =
                take_lasm_internal_header_value(response, LASM_INTERNAL_DB_HANDLE_HEADER)
            else {
                set_lasm_json_response(
                    response,
                    400,
                    &lasm_error_envelope(
                        "DB.EXEC_INVALID",
                        "validation",
                        "db.exec requires db capability and query handle",
                        400,
                        trace_id,
                    ),
                );
                return true;
            };
            let db_raw =
                materialize_lasm_internal_header_value(raw_db_header, request, path_params);
            let db = match parse_lasm_positive_i64(db_raw.trim()) {
                Some(value) => value,
                None => {
                    set_lasm_json_response(
                        response,
                        400,
                        &lasm_error_envelope(
                            "DB.EXEC_INVALID",
                            "validation",
                            "db.exec requires db capability and query handle",
                            400,
                            trace_id,
                        ),
                    );
                    return true;
                }
            };
            if !is_lasm_valid_db_cap_handle(db) {
                set_lasm_json_response(
                    response,
                    400,
                    &lasm_error_envelope(
                        "DB.EXEC_INVALID",
                        "validation",
                        "db.exec requires db capability and query handle",
                        400,
                        trace_id,
                    ),
                );
                return true;
            }
            let template = template.trim().to_string();
            let (params, parsed_params) = normalize_lasm_db_params_and_value(params.as_str());
            if !enforce_lasm_db_params_max_entries(
                response,
                "exec",
                parsed_params.as_ref(),
                params.as_str(),
                trace_id,
            ) {
                return true;
            }
            let prepared_params = match prepare_lasm_db_operation_params(
                db_records_adapter,
                "DB.EXEC_INVALID",
                template.as_str(),
                params.as_str(),
                parsed_params.as_ref(),
                response,
                trace_id,
            ) {
                Some(value) => value,
                None => return true,
            };
            if db_records_adapter == LasmDbRecordsAdapter::Postgres {
                let (postgres_template, postgres_params) = prepared_params
                    .postgres
                    .as_ref()
                    .expect("postgres preparse should exist for postgres adapter path");
                let postgres_config = {
                    let state =
                        match lock_lasm_dynamic_state_or_respond(dynamic_state, response, trace_id)
                        {
                            Some(state) => state,
                            None => return true,
                        };
                    debug_assert_eq!(state.db_records_adapter, db_records_adapter);
                    match build_lasm_postgres_thread_local_config(&state) {
                        Ok(config) => config,
                        Err(message) => {
                            set_lasm_db_runtime_error_response(
                                response,
                                "exec",
                                message.as_str(),
                                trace_id,
                            );
                            return true;
                        }
                    }
                };
                let affected_rows = match run_lasm_postgres_exec_thread_local(
                    &postgres_config,
                    postgres_template.as_str(),
                    postgres_params.as_slice(),
                ) {
                    Ok(value) => value,
                    Err(message) => {
                        set_lasm_db_runtime_error_response(
                            response,
                            "exec",
                            message.as_str(),
                            trace_id,
                        );
                        return true;
                    }
                };
                let (record, compaction_snapshot) = {
                    let mut state =
                        match lock_lasm_dynamic_state_or_respond(dynamic_state, response, trace_id)
                        {
                            Some(state) => state,
                            None => return true,
                        };
                    let record = LasmDbRecord {
                        id: state.next_db_record_id,
                        op: "exec".to_string(),
                        db,
                        template: template.clone(),
                        params: params.clone(),
                        tx: 0,
                        affected_rows,
                        created_at_ms: lasm_now_ms(),
                    };
                    state.next_db_record_id = state.next_db_record_id.saturating_add(1);
                    append_lasm_db_record_in_memory_with_compaction_snapshot(&mut state, record)
                };
                persist_lasm_postgres_record_after_unlock(
                    &postgres_config,
                    &record,
                    compaction_snapshot,
                );
                set_lasm_json_response(
                    response,
                    200,
                    &serde_json::json!({
                        "ok": true,
                        "recordId": record.id,
                        "db": record.db,
                        "op": record.op,
                        "template": record.template,
                        "params": record.params,
                        "tx": record.tx,
                        "affectedRows": affected_rows,
                    }),
                );
                return true;
            }
            let (record, affected_rows) = {
                let mut state =
                    match lock_lasm_dynamic_state_or_respond(dynamic_state, response, trace_id) {
                        Some(state) => state,
                        None => return true,
                    };
                debug_assert_eq!(state.db_records_adapter, db_records_adapter);
                let mut affected_rows = 0u64;
                if db_records_adapter == LasmDbRecordsAdapter::Sqlite {
                    let sqlite_params = prepared_params
                        .sqlite
                        .as_ref()
                        .expect("sqlite params should exist for sqlite adapter path");
                    let sqlite_affected_rows =
                        match run_lasm_sqlite_exec(&mut state, template.as_str(), sqlite_params) {
                            Ok(value) => value,
                            Err(message) => {
                                set_lasm_db_runtime_error_response(
                                    response,
                                    "exec",
                                    message.as_str(),
                                    trace_id,
                                );
                                return true;
                            }
                        };
                    affected_rows = sqlite_affected_rows;
                }
                let record = LasmDbRecord {
                    id: state.next_db_record_id,
                    op: "exec".to_string(),
                    db,
                    template: template.clone(),
                    params: params.clone(),
                    tx: 0,
                    affected_rows,
                    created_at_ms: lasm_now_ms(),
                };
                state.next_db_record_id = state.next_db_record_id.saturating_add(1);
                persist_lasm_db_record_with_capacity_guard(&mut state, &record);
                (record, affected_rows)
            };
            set_lasm_json_response(
                response,
                200,
                &serde_json::json!({
                    "ok": true,
                    "recordId": record.id,
                    "db": record.db,
                    "op": record.op,
                    "template": record.template,
                    "params": record.params,
                    "tx": record.tx,
                    "affectedRows": affected_rows,
                }),
            );
            true
        }
        "execTx" => {
            let Some(raw_template_header) =
                take_lasm_internal_header_value(response, LASM_INTERNAL_DB_TEMPLATE_HEADER)
            else {
                set_lasm_json_response(
                    response,
                    400,
                    &lasm_error_envelope(
                        "DB.EXEC_TX_INVALID",
                        "validation",
                        "db.execTx requires transaction and query handles",
                        400,
                        trace_id,
                    ),
                );
                return true;
            };
            let template =
                materialize_lasm_internal_header_value(raw_template_header, request, path_params);
            if !enforce_lasm_db_sql_template_max_bytes(
                response,
                "execTx",
                template.as_str(),
                trace_id,
            ) {
                return true;
            }
            if template.trim().is_empty() {
                set_lasm_json_response(
                    response,
                    400,
                    &lasm_error_envelope(
                        "DB.SQL_TEMPLATE_INVALID",
                        "validation",
                        "sql.q query template is required",
                        400,
                        trace_id,
                    ),
                );
                return true;
            }
            let Some(raw_params_header) =
                take_lasm_internal_header_value(response, LASM_INTERNAL_DB_PARAMS_HEADER)
            else {
                set_lasm_json_response(
                    response,
                    400,
                    &lasm_error_envelope(
                        "DB.EXEC_TX_INVALID",
                        "validation",
                        "db.execTx requires transaction and query handles",
                        400,
                        trace_id,
                    ),
                );
                return true;
            };
            let params =
                materialize_lasm_internal_header_value(raw_params_header, request, path_params);
            if !enforce_lasm_db_params_required(response, "execTx", params.as_str(), trace_id) {
                return true;
            }
            if !enforce_lasm_db_params_max_bytes(response, "execTx", params.as_str(), trace_id) {
                return true;
            }
            let keep_allocated_tx_handle = take_lasm_internal_header_value(
                response,
                LASM_INTERNAL_DB_TX_SEQUENCE_RETAIN_HEADER,
            )
            .map(|value| value.trim() == "1")
            .unwrap_or(false);
            let tx_db_source =
                take_lasm_internal_header_value(response, LASM_INTERNAL_DB_TX_DB_HEADER).map(
                    |value| materialize_lasm_internal_header_value(value, request, path_params),
                );
            let tx_handle_raw =
                take_lasm_internal_header_value(response, LASM_INTERNAL_DB_TX_HEADER).map(
                    |value| materialize_lasm_internal_header_value(value, request, path_params),
                );
            let template = template.trim().to_string();
            let (params, parsed_params) = normalize_lasm_db_params_and_value(params.as_str());
            if !enforce_lasm_db_params_max_entries(
                response,
                "execTx",
                parsed_params.as_ref(),
                params.as_str(),
                trace_id,
            ) {
                return true;
            }
            let tx_source = match resolve_lasm_exec_tx_source(
                tx_db_source.as_deref(),
                tx_handle_raw.as_deref(),
                response,
                trace_id,
            ) {
                Some(value) => value,
                None => return true,
            };
            let prepared_params = match prepare_lasm_db_operation_params(
                db_records_adapter,
                "DB.EXEC_TX_INVALID",
                template.as_str(),
                params.as_str(),
                parsed_params.as_ref(),
                response,
                trace_id,
            ) {
                Some(value) => value,
                None => return true,
            };
            if db_records_adapter == LasmDbRecordsAdapter::Postgres {
                let (postgres_template, postgres_params) = prepared_params
                    .postgres
                    .as_ref()
                    .expect("postgres preparse should exist for postgres adapter path");
                let mut state =
                    match lock_lasm_dynamic_state_or_respond(dynamic_state, response, trace_id) {
                        Some(state) => state,
                        None => return true,
                    };
                debug_assert_eq!(state.db_records_adapter, db_records_adapter);
                let (db, tx, allocated_tx_handle) = match resolve_lasm_exec_tx_state_bindings(
                    &mut state, &tx_source, response, trace_id,
                ) {
                    Some(value) => value,
                    None => return true,
                };
                let postgres_config = match build_lasm_postgres_thread_local_config(&state) {
                    Ok(config) => config,
                    Err(message) => {
                        if let Some(tx_handle) = allocated_tx_handle {
                            state.db_tx_handles.remove(&tx_handle);
                        }
                        set_lasm_db_runtime_error_response(
                            response,
                            "execTx",
                            message.as_str(),
                            trace_id,
                        );
                        return true;
                    }
                };
                drop(state);
                let affected_rows = match run_lasm_postgres_exec_tx_thread_local(
                    &postgres_config,
                    postgres_template.as_str(),
                    postgres_params.as_slice(),
                ) {
                    Ok(value) => value,
                    Err(message) => {
                        cleanup_lasm_exec_tx_allocated_handle(dynamic_state, allocated_tx_handle);
                        set_lasm_db_runtime_error_response(
                            response,
                            "execTx",
                            message.as_str(),
                            trace_id,
                        );
                        return true;
                    }
                };
                let (record, compaction_snapshot) = {
                    let mut state =
                        match lock_lasm_dynamic_state_or_respond(dynamic_state, response, trace_id)
                        {
                            Some(state) => state,
                            None => return true,
                        };
                    if let Some(tx_handle) = allocated_tx_handle {
                        if !keep_allocated_tx_handle {
                            state.db_tx_handles.remove(&tx_handle);
                        }
                    }
                    let record = LasmDbRecord {
                        id: state.next_db_record_id,
                        op: "execTx".to_string(),
                        db,
                        template: template.clone(),
                        params: params.clone(),
                        tx,
                        affected_rows,
                        created_at_ms: lasm_now_ms(),
                    };
                    state.next_db_record_id = state.next_db_record_id.saturating_add(1);
                    append_lasm_db_record_in_memory_with_compaction_snapshot(&mut state, record)
                };
                persist_lasm_postgres_record_after_unlock(
                    &postgres_config,
                    &record,
                    compaction_snapshot,
                );
                set_lasm_json_response(
                    response,
                    200,
                    &serde_json::json!({
                        "ok": true,
                        "recordId": record.id,
                        "db": record.db,
                        "op": record.op,
                        "template": record.template,
                        "params": record.params,
                        "tx": record.tx,
                        "affectedRows": affected_rows,
                    }),
                );
                response.headers.insert(
                    LASM_INTERNAL_DB_TX_RESULT_HEADER.to_string(),
                    record.tx.to_string(),
                );
                return true;
            }

            let (record, affected_rows) = {
                let mut state =
                    match lock_lasm_dynamic_state_or_respond(dynamic_state, response, trace_id) {
                        Some(state) => state,
                        None => return true,
                    };
                debug_assert_eq!(state.db_records_adapter, db_records_adapter);
                let (db, tx, allocated_tx_handle) = match resolve_lasm_exec_tx_state_bindings(
                    &mut state, &tx_source, response, trace_id,
                ) {
                    Some(value) => value,
                    None => return true,
                };
                let mut affected_rows = 0u64;
                if db_records_adapter == LasmDbRecordsAdapter::Sqlite {
                    let sqlite_params = prepared_params
                        .sqlite
                        .as_ref()
                        .expect("sqlite params should exist for sqlite adapter path");
                    let sqlite_affected_rows =
                        match run_lasm_sqlite_exec_tx(&mut state, template.as_str(), sqlite_params)
                        {
                            Ok(value) => value,
                            Err(message) => {
                                if let Some(tx_handle) = allocated_tx_handle {
                                    state.db_tx_handles.remove(&tx_handle);
                                }
                                set_lasm_db_runtime_error_response(
                                    response,
                                    "execTx",
                                    message.as_str(),
                                    trace_id,
                                );
                                return true;
                            }
                        };
                    affected_rows = sqlite_affected_rows;
                }
                if let Some(tx_handle) = allocated_tx_handle {
                    if !keep_allocated_tx_handle {
                        state.db_tx_handles.remove(&tx_handle);
                    }
                }
                let record = LasmDbRecord {
                    id: state.next_db_record_id,
                    op: "execTx".to_string(),
                    db,
                    template: template.clone(),
                    params: params.clone(),
                    tx,
                    affected_rows,
                    created_at_ms: lasm_now_ms(),
                };
                state.next_db_record_id = state.next_db_record_id.saturating_add(1);
                persist_lasm_db_record_with_capacity_guard(&mut state, &record);
                (record, affected_rows)
            };
            set_lasm_json_response(
                response,
                200,
                &serde_json::json!({
                    "ok": true,
                    "recordId": record.id,
                    "db": record.db,
                    "op": record.op,
                    "template": record.template,
                    "params": record.params,
                    "tx": record.tx,
                    "affectedRows": affected_rows,
                }),
            );
            response.headers.insert(
                LASM_INTERNAL_DB_TX_RESULT_HEADER.to_string(),
                record.tx.to_string(),
            );
            true
        }
        "queryOne" => {
            let Some(raw_template_header) =
                take_lasm_internal_header_value(response, LASM_INTERNAL_DB_TEMPLATE_HEADER)
            else {
                set_lasm_json_response(
                    response,
                    400,
                    &lasm_error_envelope(
                        "DB.QUERY_ONE_INVALID",
                        "validation",
                        "db.queryOne requires db capability, query, and row schema handles",
                        400,
                        trace_id,
                    ),
                );
                return true;
            };
            let template =
                materialize_lasm_internal_header_value(raw_template_header, request, path_params);
            if !enforce_lasm_db_sql_template_max_bytes(
                response,
                "queryOne",
                template.as_str(),
                trace_id,
            ) {
                return true;
            }
            if template.trim().is_empty() {
                set_lasm_json_response(
                    response,
                    400,
                    &lasm_error_envelope(
                        "DB.SQL_TEMPLATE_INVALID",
                        "validation",
                        "sql.q query template is required",
                        400,
                        trace_id,
                    ),
                );
                return true;
            }
            let Some(raw_params_header) =
                take_lasm_internal_header_value(response, LASM_INTERNAL_DB_PARAMS_HEADER)
            else {
                set_lasm_json_response(
                    response,
                    400,
                    &lasm_error_envelope(
                        "DB.QUERY_ONE_INVALID",
                        "validation",
                        "db.queryOne requires db capability, query, and row schema handles",
                        400,
                        trace_id,
                    ),
                );
                return true;
            };
            let params =
                materialize_lasm_internal_header_value(raw_params_header, request, path_params);
            if !enforce_lasm_db_params_required(response, "queryOne", params.as_str(), trace_id) {
                return true;
            }
            if !enforce_lasm_db_params_max_bytes(response, "queryOne", params.as_str(), trace_id) {
                return true;
            }
            let Some(raw_db_header) =
                take_lasm_internal_header_value(response, LASM_INTERNAL_DB_HANDLE_HEADER)
            else {
                set_lasm_json_response(
                    response,
                    400,
                    &lasm_error_envelope(
                        "DB.QUERY_ONE_INVALID",
                        "validation",
                        "db.queryOne requires db capability, query, and row schema handles",
                        400,
                        trace_id,
                    ),
                );
                return true;
            };
            let db_raw =
                materialize_lasm_internal_header_value(raw_db_header, request, path_params);
            let db = match parse_lasm_positive_i64(db_raw.trim()) {
                Some(value) => value,
                None => {
                    set_lasm_json_response(
                        response,
                        400,
                        &lasm_error_envelope(
                            "DB.QUERY_ONE_INVALID",
                            "validation",
                            "db.queryOne requires db capability, query, and row schema handles",
                            400,
                            trace_id,
                        ),
                    );
                    return true;
                }
            };
            if !is_lasm_valid_db_cap_handle(db) {
                set_lasm_json_response(
                    response,
                    400,
                    &lasm_error_envelope(
                        "DB.QUERY_ONE_INVALID",
                        "validation",
                        "db.queryOne requires db capability, query, and row schema handles",
                        400,
                        trace_id,
                    ),
                );
                return true;
            }
            let Some(raw_row_schema_header) =
                take_lasm_internal_header_value(response, LASM_INTERNAL_DB_ROW_SCHEMA_HEADER)
            else {
                set_lasm_json_response(
                    response,
                    400,
                    &lasm_error_envelope(
                        "DB.QUERY_ONE_INVALID",
                        "validation",
                        "db.queryOne requires db capability, query, and row schema handles",
                        400,
                        trace_id,
                    ),
                );
                return true;
            };
            let row_schema_raw =
                materialize_lasm_internal_header_value(raw_row_schema_header, request, path_params);
            let row_schema = match parse_lasm_positive_i64(row_schema_raw.trim()) {
                Some(value) => value,
                None => {
                    set_lasm_json_response(
                        response,
                        400,
                        &lasm_error_envelope(
                            "DB.QUERY_ONE_INVALID",
                            "validation",
                            "db.queryOne requires db capability, query, and row schema handles",
                            400,
                            trace_id,
                        ),
                    );
                    return true;
                }
            };
            let template = template.trim().to_string();
            let (params, parsed_params) = normalize_lasm_db_params_and_value(params.as_str());
            if !enforce_lasm_db_params_max_entries(
                response,
                "queryOne",
                parsed_params.as_ref(),
                params.as_str(),
                trace_id,
            ) {
                return true;
            }
            let prepared_params = match prepare_lasm_db_operation_params(
                db_records_adapter,
                "DB.QUERY_ONE_INVALID",
                template.as_str(),
                params.as_str(),
                parsed_params.as_ref(),
                response,
                trace_id,
            ) {
                Some(value) => value,
                None => return true,
            };
            if db_records_adapter == LasmDbRecordsAdapter::Postgres {
                let (postgres_template, postgres_params) = prepared_params
                    .postgres
                    .as_ref()
                    .expect("postgres preparse should exist for postgres adapter path");
                let state =
                    match lock_lasm_dynamic_state_or_respond(dynamic_state, response, trace_id) {
                        Some(state) => state,
                        None => return true,
                    };
                debug_assert_eq!(state.db_records_adapter, db_records_adapter);
                let postgres_config = match build_lasm_postgres_thread_local_config(&state) {
                    Ok(config) => config,
                    Err(message) => {
                        set_lasm_db_runtime_error_response(
                            response,
                            "queryOne",
                            message.as_str(),
                            trace_id,
                        );
                        return true;
                    }
                };
                drop(state);
                let row_object = match run_lasm_postgres_query_one_thread_local(
                    &postgres_config,
                    postgres_template.as_str(),
                    postgres_params.as_slice(),
                ) {
                    Ok(Some(value)) => value,
                    Ok(None) => {
                        set_lasm_json_response(
                            response,
                            404,
                            &lasm_error_envelope(
                                "DB.QUERY_ONE_NOT_FOUND",
                                "missing_dependency",
                                "db.queryOne row not found",
                                404,
                                trace_id,
                            ),
                        );
                        return true;
                    }
                    Err(message) => {
                        set_lasm_db_runtime_error_response(
                            response,
                            "queryOne",
                            message.as_str(),
                            trace_id,
                        );
                        return true;
                    }
                };
                let row = serde_json::to_string(&row_object).unwrap_or_else(|_| "{}".to_string());
                if !enforce_lasm_db_query_one_row_max_columns(response, &row_object, trace_id) {
                    return true;
                }
                if !enforce_lasm_db_query_one_row_max_bytes(response, row.as_str(), trace_id) {
                    return true;
                }
                let (record, compaction_snapshot) = {
                    let mut state =
                        match lock_lasm_dynamic_state_or_respond(dynamic_state, response, trace_id)
                        {
                            Some(state) => state,
                            None => return true,
                        };
                    let record = LasmDbRecord {
                        id: state.next_db_record_id,
                        op: "queryOne".to_string(),
                        db,
                        template: template.clone(),
                        params: params.clone(),
                        tx: 0,
                        affected_rows: 1,
                        created_at_ms: lasm_now_ms(),
                    };
                    state.next_db_record_id = state.next_db_record_id.saturating_add(1);
                    append_lasm_db_record_in_memory_with_compaction_snapshot(&mut state, record)
                };
                persist_lasm_postgres_record_after_unlock(
                    &postgres_config,
                    &record,
                    compaction_snapshot,
                );
                set_lasm_json_response(
                    response,
                    200,
                    &serde_json::json!({
                        "ok": true,
                        "recordId": record.id,
                        "rowSchema": row_schema,
                        "row": row,
                        "rowObject": row_object,
                        "record": lasm_db_record_to_json(&record),
                    }),
                );
                return true;
            }
            let matched_record = {
                let mut state =
                    match lock_lasm_dynamic_state_or_respond(dynamic_state, response, trace_id) {
                        Some(state) => state,
                        None => return true,
                    };
                debug_assert_eq!(state.db_records_adapter, db_records_adapter);
                if db_records_adapter == LasmDbRecordsAdapter::Sqlite {
                    let sqlite_params = prepared_params
                        .sqlite
                        .as_ref()
                        .expect("sqlite params should exist for sqlite adapter path");
                    let row_object = match run_lasm_sqlite_query_one(
                        &mut state,
                        template.as_str(),
                        sqlite_params,
                    ) {
                        Ok(Some(value)) => value,
                        Ok(None) => {
                            set_lasm_json_response(
                                response,
                                404,
                                &lasm_error_envelope(
                                    "DB.QUERY_ONE_NOT_FOUND",
                                    "missing_dependency",
                                    "db.queryOne row not found",
                                    404,
                                    trace_id,
                                ),
                            );
                            return true;
                        }
                        Err(message) => {
                            set_lasm_db_runtime_error_response(
                                response,
                                "queryOne",
                                message.as_str(),
                                trace_id,
                            );
                            return true;
                        }
                    };
                    let row =
                        serde_json::to_string(&row_object).unwrap_or_else(|_| "{}".to_string());
                    if !enforce_lasm_db_query_one_row_max_columns(response, &row_object, trace_id) {
                        return true;
                    }
                    if !enforce_lasm_db_query_one_row_max_bytes(response, row.as_str(), trace_id) {
                        return true;
                    }
                    let record = LasmDbRecord {
                        id: state.next_db_record_id,
                        op: "queryOne".to_string(),
                        db,
                        template: template.clone(),
                        params: params.clone(),
                        tx: 0,
                        affected_rows: 1,
                        created_at_ms: lasm_now_ms(),
                    };
                    state.next_db_record_id = state.next_db_record_id.saturating_add(1);
                    persist_lasm_db_record_with_capacity_guard(&mut state, &record);
                    set_lasm_json_response(
                        response,
                        200,
                        &serde_json::json!({
                            "ok": true,
                            "recordId": record.id,
                            "rowSchema": row_schema,
                            "row": row,
                            "rowObject": row_object,
                            "record": lasm_db_record_to_json(&record),
                        }),
                    );
                    return true;
                }
                let matched_source_record = find_lasm_records_log_latest_match(
                    &state,
                    db,
                    template.as_str(),
                    params.as_str(),
                );
                if let Some(matched_source_record) = matched_source_record {
                    let record = LasmDbRecord {
                        id: state.next_db_record_id,
                        op: "queryOne".to_string(),
                        db,
                        template: template.clone(),
                        params: params.clone(),
                        tx: 0,
                        affected_rows: 1,
                        created_at_ms: lasm_now_ms(),
                    };
                    state.next_db_record_id = state.next_db_record_id.saturating_add(1);
                    persist_lasm_db_record_with_capacity_guard(&mut state, &record);
                    Some((record, matched_source_record))
                } else {
                    None
                }
            };
            let Some((record, matched_source_record)) = matched_record else {
                set_lasm_json_response(
                    response,
                    404,
                    &lasm_error_envelope(
                        "DB.QUERY_ONE_NOT_FOUND",
                        "missing_dependency",
                        "db.queryOne record not found",
                        404,
                        trace_id,
                    ),
                );
                return true;
            };
            let row_object =
                build_lasm_records_log_query_one_row_object(&matched_source_record, row_schema);
            let row = serde_json::to_string(&row_object).unwrap_or_else(|_| "{}".to_string());
            if !enforce_lasm_db_query_one_row_max_columns(response, &row_object, trace_id) {
                return true;
            }
            if !enforce_lasm_db_query_one_row_max_bytes(response, row.as_str(), trace_id) {
                return true;
            }
            set_lasm_json_response(
                response,
                200,
                &serde_json::json!({
                    "ok": true,
                    "recordId": record.id,
                    "rowSchema": row_schema,
                    "row": row,
                    "rowObject": row_object,
                    "record": lasm_db_record_to_json(&record),
                }),
            );
            true
        }
        _ => {
            set_lasm_json_response(
                response,
                400,
                &lasm_error_envelope(
                    "DB.OPERATION_INVALID",
                    "validation",
                    "unsupported internal db operation marker",
                    400,
                    trace_id,
                ),
            );
            true
        }
    }
}

fn cleanup_lasm_internal_db_sequence_tx_handles(
    dynamic_state: &Mutex<LasmDynamicResponseState>,
    handles: impl IntoIterator<Item = i64>,
) {
    let Ok(mut state) = dynamic_state.lock() else {
        return;
    };
    for handle in handles {
        state.db_tx_handles.remove(&handle);
    }
}

fn take_lasm_internal_header_value(
    response: &mut sec4_core::HttpResponse,
    header_name: &str,
) -> Option<String> {
    let key = crate::find_lasm_header_key_case_insensitive(&response.headers, header_name)?;
    response.headers.remove(&key)
}

fn take_lasm_internal_header_value_indexed(
    response: &mut sec4_core::HttpResponse,
    header_name: &str,
    index: usize,
) -> Option<String> {
    let indexed = lasm_internal_db_indexed_header(header_name, index);
    let key = crate::find_lasm_header_key_case_insensitive(&response.headers, indexed.as_str())?;
    response.headers.remove(&key)
}

fn materialize_lasm_internal_header_value(
    value: String,
    request: &LasmRunRequest,
    path_params: &BTreeMap<String, String>,
) -> String {
    if crate::contains_lasm_request_placeholder_tokens(value.as_str()) {
        crate::materialize_lasm_request_placeholders(value.as_str(), request, path_params)
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use super::apply_lasm_internal_db_operation_materialization;
    use crate::{
        lasm_internal_db_indexed_header, LasmDbRecord, LasmDbRecordsAdapter,
        LasmDynamicResponseState, LasmRunRequest, LASM_INTERNAL_DB_HANDLE_HEADER,
        LASM_INTERNAL_DB_OP_COUNT_HEADER, LASM_INTERNAL_DB_OP_HEADER,
        LASM_INTERNAL_DB_PARAMS_HEADER, LASM_INTERNAL_DB_ROW_SCHEMA_HEADER,
        LASM_INTERNAL_DB_TEMPLATE_HEADER, LASM_INTERNAL_DB_TX_DB_HEADER,
        LASM_INTERNAL_DB_TX_HEADER,
    };
    use std::collections::BTreeMap;
    use std::sync::Mutex;

    fn empty_request() -> LasmRunRequest {
        LasmRunRequest {
            method: "GET".to_string(),
            http_version: "HTTP/1.1".to_string(),
            path: "/".to_string(),
            query_params: BTreeMap::new(),
            headers: BTreeMap::new(),
            body: Vec::new(),
        }
    }

    #[test]
    fn rejects_unknown_internal_db_operation_marker() {
        let request = empty_request();
        let path_params = BTreeMap::new();
        let dynamic_state = Mutex::new(LasmDynamicResponseState::default());
        let mut response = sec4_core::HttpResponse::text(200, "");
        response
            .headers
            .insert(LASM_INTERNAL_DB_OP_HEADER.to_string(), "bogus".to_string());

        let handled = apply_lasm_internal_db_operation_materialization(
            &mut response,
            &request,
            &path_params,
            &dynamic_state,
            LasmDbRecordsAdapter::RecordsLog,
            "rt-unit",
        );

        assert!(
            handled,
            "invalid marker should be handled deterministically"
        );
        assert_eq!(response.status, 400);
        let body = String::from_utf8(response.body).expect("response body should be utf-8 JSON");
        assert!(body.contains("\"code\":\"DB.OPERATION_INVALID\""));
    }

    #[test]
    fn rejects_invalid_internal_db_operation_count_marker() {
        let request = empty_request();
        let path_params = BTreeMap::new();
        let dynamic_state = Mutex::new(LasmDynamicResponseState::default());
        let mut response = sec4_core::HttpResponse::text(200, "");
        response.headers.insert(
            LASM_INTERNAL_DB_OP_COUNT_HEADER.to_string(),
            "abc".to_string(),
        );

        let handled = apply_lasm_internal_db_operation_materialization(
            &mut response,
            &request,
            &path_params,
            &dynamic_state,
            LasmDbRecordsAdapter::RecordsLog,
            "rt-unit",
        );

        assert!(
            handled,
            "invalid operation-count marker should be handled deterministically"
        );
        assert_eq!(response.status, 400);
        let body = String::from_utf8(response.body).expect("response body should be utf-8 JSON");
        assert!(body.contains("\"code\":\"DB.OPERATION_INVALID\""));
        assert!(body.contains("invalid internal db operation sequence marker"));
    }

    #[test]
    fn rejects_single_value_internal_db_operation_count_marker() {
        let request = empty_request();
        let path_params = BTreeMap::new();
        let dynamic_state = Mutex::new(LasmDynamicResponseState::default());
        let mut response = sec4_core::HttpResponse::text(200, "");
        response.headers.insert(
            LASM_INTERNAL_DB_OP_COUNT_HEADER.to_string(),
            "1".to_string(),
        );

        let handled = apply_lasm_internal_db_operation_materialization(
            &mut response,
            &request,
            &path_params,
            &dynamic_state,
            LasmDbRecordsAdapter::RecordsLog,
            "rt-unit",
        );

        assert!(
            handled,
            "single-value operation-count marker should be handled deterministically"
        );
        assert_eq!(response.status, 400);
        let body = String::from_utf8(response.body).expect("response body should be utf-8 JSON");
        assert!(body.contains("\"code\":\"DB.OPERATION_INVALID\""));
        assert!(body.contains("internal db operation sequence marker value must be >= 2"));
    }

    #[test]
    fn rejects_indexed_internal_db_markers_without_operation_count() {
        let request = empty_request();
        let path_params = BTreeMap::new();
        let dynamic_state = Mutex::new(LasmDynamicResponseState::default());
        let mut response = sec4_core::HttpResponse::text(200, "");
        response.headers.insert(
            lasm_internal_db_indexed_header(LASM_INTERNAL_DB_OP_HEADER, 0),
            "exec".to_string(),
        );

        let handled = apply_lasm_internal_db_operation_materialization(
            &mut response,
            &request,
            &path_params,
            &dynamic_state,
            LasmDbRecordsAdapter::RecordsLog,
            "rt-unit",
        );

        assert!(
            handled,
            "indexed markers without op-count should be handled deterministically"
        );
        assert_eq!(response.status, 400);
        let body = String::from_utf8(response.body).expect("response body should be utf-8 JSON");
        assert!(body.contains("\"code\":\"DB.OPERATION_INVALID\""));
        assert!(
            body.contains("indexed internal db operation markers require operation count marker")
        );
    }

    #[test]
    fn list_records_marker_materializes_records_payload() {
        let request = empty_request();
        let path_params = BTreeMap::new();
        let mut state = LasmDynamicResponseState::default();
        state.db_records.push(LasmDbRecord {
            id: 1,
            op: "exec".to_string(),
            db: 1,
            template: "SELECT 1".to_string(),
            params: "[1]".to_string(),
            tx: 0,
            affected_rows: 1,
            created_at_ms: 1,
        });
        let dynamic_state = Mutex::new(state);
        let mut response = sec4_core::HttpResponse::text(200, "");
        response.headers.insert(
            LASM_INTERNAL_DB_OP_HEADER.to_string(),
            "listRecords".to_string(),
        );

        let handled = apply_lasm_internal_db_operation_materialization(
            &mut response,
            &request,
            &path_params,
            &dynamic_state,
            LasmDbRecordsAdapter::RecordsLog,
            "rt-unit",
        );

        assert!(handled, "listRecords marker should be handled");
        assert_eq!(response.status, 200);
        let body = String::from_utf8(response.body).expect("response body should be utf-8 JSON");
        assert!(body.contains("\"count\":1"));
        assert!(body.contains("\"recordsTotal\":1"));
        assert!(body.contains("\"op\":\"exec\""));
    }

    #[test]
    fn exec_marker_rejects_missing_db_handle_header() {
        let request = empty_request();
        let path_params = BTreeMap::new();
        let dynamic_state = Mutex::new(LasmDynamicResponseState::default());
        let mut response = sec4_core::HttpResponse::text(200, "");
        response
            .headers
            .insert(LASM_INTERNAL_DB_OP_HEADER.to_string(), "exec".to_string());
        response.headers.insert(
            LASM_INTERNAL_DB_TEMPLATE_HEADER.to_string(),
            "SELECT 1".to_string(),
        );
        response
            .headers
            .insert(LASM_INTERNAL_DB_PARAMS_HEADER.to_string(), "[]".to_string());

        let handled = apply_lasm_internal_db_operation_materialization(
            &mut response,
            &request,
            &path_params,
            &dynamic_state,
            LasmDbRecordsAdapter::RecordsLog,
            "rt-unit",
        );

        assert!(
            handled,
            "missing db handle should be handled deterministically"
        );
        assert_eq!(response.status, 400);
        let body = String::from_utf8(response.body).expect("response body should be utf-8 JSON");
        assert!(body.contains("\"code\":\"DB.EXEC_INVALID\""));
    }

    #[test]
    fn exec_marker_rejects_missing_params_header() {
        let request = empty_request();
        let path_params = BTreeMap::new();
        let dynamic_state = Mutex::new(LasmDynamicResponseState::default());
        let mut response = sec4_core::HttpResponse::text(200, "");
        response
            .headers
            .insert(LASM_INTERNAL_DB_OP_HEADER.to_string(), "exec".to_string());
        response
            .headers
            .insert(LASM_INTERNAL_DB_HANDLE_HEADER.to_string(), "1".to_string());
        response.headers.insert(
            LASM_INTERNAL_DB_TEMPLATE_HEADER.to_string(),
            "SELECT 1".to_string(),
        );

        let handled = apply_lasm_internal_db_operation_materialization(
            &mut response,
            &request,
            &path_params,
            &dynamic_state,
            LasmDbRecordsAdapter::RecordsLog,
            "rt-unit",
        );

        assert!(
            handled,
            "missing params marker should be handled deterministically"
        );
        assert_eq!(response.status, 400);
        let body = String::from_utf8(response.body).expect("response body should be utf-8 JSON");
        assert!(body.contains("\"code\":\"DB.EXEC_INVALID\""));
    }

    #[test]
    fn exec_marker_rejects_missing_template_header() {
        let request = empty_request();
        let path_params = BTreeMap::new();
        let dynamic_state = Mutex::new(LasmDynamicResponseState::default());
        let mut response = sec4_core::HttpResponse::text(200, "");
        response
            .headers
            .insert(LASM_INTERNAL_DB_OP_HEADER.to_string(), "exec".to_string());
        response
            .headers
            .insert(LASM_INTERNAL_DB_HANDLE_HEADER.to_string(), "1".to_string());
        response
            .headers
            .insert(LASM_INTERNAL_DB_PARAMS_HEADER.to_string(), "[]".to_string());

        let handled = apply_lasm_internal_db_operation_materialization(
            &mut response,
            &request,
            &path_params,
            &dynamic_state,
            LasmDbRecordsAdapter::RecordsLog,
            "rt-unit",
        );

        assert!(
            handled,
            "missing template marker should be handled deterministically"
        );
        assert_eq!(response.status, 400);
        let body = String::from_utf8(response.body).expect("response body should be utf-8 JSON");
        assert!(body.contains("\"code\":\"DB.EXEC_INVALID\""));
    }

    #[test]
    fn query_one_marker_rejects_missing_row_schema_header() {
        let request = empty_request();
        let path_params = BTreeMap::new();
        let dynamic_state = Mutex::new(LasmDynamicResponseState::default());
        let mut response = sec4_core::HttpResponse::text(200, "");
        response.headers.insert(
            LASM_INTERNAL_DB_OP_HEADER.to_string(),
            "queryOne".to_string(),
        );
        response
            .headers
            .insert(LASM_INTERNAL_DB_HANDLE_HEADER.to_string(), "1".to_string());
        response.headers.insert(
            LASM_INTERNAL_DB_TEMPLATE_HEADER.to_string(),
            "SELECT 1".to_string(),
        );
        response
            .headers
            .insert(LASM_INTERNAL_DB_PARAMS_HEADER.to_string(), "[]".to_string());

        let handled = apply_lasm_internal_db_operation_materialization(
            &mut response,
            &request,
            &path_params,
            &dynamic_state,
            LasmDbRecordsAdapter::RecordsLog,
            "rt-unit",
        );

        assert!(
            handled,
            "missing queryOne row schema should be handled deterministically"
        );
        assert_eq!(response.status, 400);
        let body = String::from_utf8(response.body).expect("response body should be utf-8 JSON");
        assert!(body.contains("\"code\":\"DB.QUERY_ONE_INVALID\""));
        assert!(
            !response
                .headers
                .contains_key(LASM_INTERNAL_DB_ROW_SCHEMA_HEADER),
            "internal row schema marker should not survive response materialization"
        );
    }

    #[test]
    fn query_one_marker_rejects_missing_params_header() {
        let request = empty_request();
        let path_params = BTreeMap::new();
        let dynamic_state = Mutex::new(LasmDynamicResponseState::default());
        let mut response = sec4_core::HttpResponse::text(200, "");
        response.headers.insert(
            LASM_INTERNAL_DB_OP_HEADER.to_string(),
            "queryOne".to_string(),
        );
        response
            .headers
            .insert(LASM_INTERNAL_DB_HANDLE_HEADER.to_string(), "1".to_string());
        response.headers.insert(
            LASM_INTERNAL_DB_ROW_SCHEMA_HEADER.to_string(),
            "7".to_string(),
        );
        response.headers.insert(
            LASM_INTERNAL_DB_TEMPLATE_HEADER.to_string(),
            "SELECT 1".to_string(),
        );

        let handled = apply_lasm_internal_db_operation_materialization(
            &mut response,
            &request,
            &path_params,
            &dynamic_state,
            LasmDbRecordsAdapter::RecordsLog,
            "rt-unit",
        );

        assert!(
            handled,
            "missing queryOne params marker should be handled deterministically"
        );
        assert_eq!(response.status, 400);
        let body = String::from_utf8(response.body).expect("response body should be utf-8 JSON");
        assert!(body.contains("\"code\":\"DB.QUERY_ONE_INVALID\""));
    }

    #[test]
    fn query_one_marker_rejects_missing_template_header() {
        let request = empty_request();
        let path_params = BTreeMap::new();
        let dynamic_state = Mutex::new(LasmDynamicResponseState::default());
        let mut response = sec4_core::HttpResponse::text(200, "");
        response.headers.insert(
            LASM_INTERNAL_DB_OP_HEADER.to_string(),
            "queryOne".to_string(),
        );
        response
            .headers
            .insert(LASM_INTERNAL_DB_HANDLE_HEADER.to_string(), "1".to_string());
        response.headers.insert(
            LASM_INTERNAL_DB_ROW_SCHEMA_HEADER.to_string(),
            "7".to_string(),
        );
        response
            .headers
            .insert(LASM_INTERNAL_DB_PARAMS_HEADER.to_string(), "[]".to_string());

        let handled = apply_lasm_internal_db_operation_materialization(
            &mut response,
            &request,
            &path_params,
            &dynamic_state,
            LasmDbRecordsAdapter::RecordsLog,
            "rt-unit",
        );

        assert!(
            handled,
            "missing queryOne template marker should be handled deterministically"
        );
        assert_eq!(response.status, 400);
        let body = String::from_utf8(response.body).expect("response body should be utf-8 JSON");
        assert!(body.contains("\"code\":\"DB.QUERY_ONE_INVALID\""));
    }

    #[test]
    fn exec_marker_rejects_empty_params_header() {
        let request = empty_request();
        let path_params = BTreeMap::new();
        let dynamic_state = Mutex::new(LasmDynamicResponseState::default());
        let mut response = sec4_core::HttpResponse::text(200, "");
        response
            .headers
            .insert(LASM_INTERNAL_DB_OP_HEADER.to_string(), "exec".to_string());
        response
            .headers
            .insert(LASM_INTERNAL_DB_HANDLE_HEADER.to_string(), "1".to_string());
        response.headers.insert(
            LASM_INTERNAL_DB_TEMPLATE_HEADER.to_string(),
            "SELECT 1".to_string(),
        );
        response.headers.insert(
            LASM_INTERNAL_DB_PARAMS_HEADER.to_string(),
            "   ".to_string(),
        );

        let handled = apply_lasm_internal_db_operation_materialization(
            &mut response,
            &request,
            &path_params,
            &dynamic_state,
            LasmDbRecordsAdapter::RecordsLog,
            "rt-unit",
        );

        assert!(
            handled,
            "empty params marker should be handled deterministically"
        );
        assert_eq!(response.status, 400);
        let body = String::from_utf8(response.body).expect("response body should be utf-8 JSON");
        assert!(body.contains("\"code\":\"DB.EXEC_INVALID\""));
        assert!(body.contains("sql.q params payload is required"));
    }

    #[test]
    fn query_one_marker_rejects_empty_params_header() {
        let request = empty_request();
        let path_params = BTreeMap::new();
        let dynamic_state = Mutex::new(LasmDynamicResponseState::default());
        let mut response = sec4_core::HttpResponse::text(200, "");
        response.headers.insert(
            LASM_INTERNAL_DB_OP_HEADER.to_string(),
            "queryOne".to_string(),
        );
        response
            .headers
            .insert(LASM_INTERNAL_DB_HANDLE_HEADER.to_string(), "1".to_string());
        response.headers.insert(
            LASM_INTERNAL_DB_ROW_SCHEMA_HEADER.to_string(),
            "7".to_string(),
        );
        response.headers.insert(
            LASM_INTERNAL_DB_TEMPLATE_HEADER.to_string(),
            "SELECT 1".to_string(),
        );
        response
            .headers
            .insert(LASM_INTERNAL_DB_PARAMS_HEADER.to_string(), " ".to_string());

        let handled = apply_lasm_internal_db_operation_materialization(
            &mut response,
            &request,
            &path_params,
            &dynamic_state,
            LasmDbRecordsAdapter::RecordsLog,
            "rt-unit",
        );

        assert!(
            handled,
            "empty queryOne params marker should be handled deterministically"
        );
        assert_eq!(response.status, 400);
        let body = String::from_utf8(response.body).expect("response body should be utf-8 JSON");
        assert!(body.contains("\"code\":\"DB.QUERY_ONE_INVALID\""));
        assert!(body.contains("sql.q params payload is required"));
    }

    #[test]
    fn exec_tx_marker_rejects_ambiguous_dual_transaction_sources() {
        let request = empty_request();
        let path_params = BTreeMap::new();
        let dynamic_state = Mutex::new(LasmDynamicResponseState::default());
        let mut response = sec4_core::HttpResponse::text(200, "");
        response
            .headers
            .insert(LASM_INTERNAL_DB_OP_HEADER.to_string(), "execTx".to_string());
        response.headers.insert(
            LASM_INTERNAL_DB_TEMPLATE_HEADER.to_string(),
            "SELECT 1".to_string(),
        );
        response
            .headers
            .insert(LASM_INTERNAL_DB_PARAMS_HEADER.to_string(), "[]".to_string());
        response
            .headers
            .insert(LASM_INTERNAL_DB_TX_DB_HEADER.to_string(), "1".to_string());
        response
            .headers
            .insert(LASM_INTERNAL_DB_TX_HEADER.to_string(), "2".to_string());

        let handled = apply_lasm_internal_db_operation_materialization(
            &mut response,
            &request,
            &path_params,
            &dynamic_state,
            LasmDbRecordsAdapter::RecordsLog,
            "rt-unit",
        );

        assert!(
            handled,
            "ambiguous execTx tx-source markers should be handled deterministically"
        );
        assert_eq!(response.status, 400);
        let body = String::from_utf8(response.body).expect("response body should be utf-8 JSON");
        assert!(body.contains("\"code\":\"DB.EXEC_TX_INVALID\""));
        assert!(body.contains("either tx handle or db.tx(dbCap) source, not both"));
    }

    #[test]
    fn exec_tx_sequence_marker_rejects_ambiguous_dual_transaction_sources() {
        let request = empty_request();
        let path_params = BTreeMap::new();
        let dynamic_state = Mutex::new(LasmDynamicResponseState::default());
        let mut response = sec4_core::HttpResponse::text(200, "");
        response.headers.insert(
            LASM_INTERNAL_DB_OP_COUNT_HEADER.to_string(),
            "2".to_string(),
        );
        response.headers.insert(
            lasm_internal_db_indexed_header(LASM_INTERNAL_DB_OP_HEADER, 0),
            "exec".to_string(),
        );
        response.headers.insert(
            lasm_internal_db_indexed_header(LASM_INTERNAL_DB_HANDLE_HEADER, 0),
            "1".to_string(),
        );
        response.headers.insert(
            lasm_internal_db_indexed_header(LASM_INTERNAL_DB_TEMPLATE_HEADER, 0),
            "SELECT 1".to_string(),
        );
        response.headers.insert(
            lasm_internal_db_indexed_header(LASM_INTERNAL_DB_PARAMS_HEADER, 0),
            "[]".to_string(),
        );
        response.headers.insert(
            lasm_internal_db_indexed_header(LASM_INTERNAL_DB_OP_HEADER, 1),
            "execTx".to_string(),
        );
        response.headers.insert(
            lasm_internal_db_indexed_header(LASM_INTERNAL_DB_TEMPLATE_HEADER, 1),
            "SELECT 2".to_string(),
        );
        response.headers.insert(
            lasm_internal_db_indexed_header(LASM_INTERNAL_DB_PARAMS_HEADER, 1),
            "[]".to_string(),
        );
        response.headers.insert(
            lasm_internal_db_indexed_header(LASM_INTERNAL_DB_TX_DB_HEADER, 1),
            "1".to_string(),
        );
        response.headers.insert(
            lasm_internal_db_indexed_header(LASM_INTERNAL_DB_TX_HEADER, 1),
            "2".to_string(),
        );

        let handled = apply_lasm_internal_db_operation_materialization(
            &mut response,
            &request,
            &path_params,
            &dynamic_state,
            LasmDbRecordsAdapter::RecordsLog,
            "rt-unit",
        );

        assert!(
            handled,
            "sequence execTx with ambiguous tx-source markers should fail deterministically"
        );
        assert_eq!(response.status, 400);
        let body = String::from_utf8(response.body).expect("response body should be utf-8 JSON");
        assert!(body.contains("\"code\":\"DB.EXEC_TX_INVALID\""));
        assert!(body.contains("either tx handle or db.tx(dbCap) source, not both"));
    }

    #[test]
    fn exec_tx_marker_rejects_missing_template_header() {
        let request = empty_request();
        let path_params = BTreeMap::new();
        let dynamic_state = Mutex::new(LasmDynamicResponseState::default());
        let mut response = sec4_core::HttpResponse::text(200, "");
        response
            .headers
            .insert(LASM_INTERNAL_DB_OP_HEADER.to_string(), "execTx".to_string());
        response
            .headers
            .insert(LASM_INTERNAL_DB_PARAMS_HEADER.to_string(), "[]".to_string());
        response
            .headers
            .insert(LASM_INTERNAL_DB_TX_DB_HEADER.to_string(), "1".to_string());

        let handled = apply_lasm_internal_db_operation_materialization(
            &mut response,
            &request,
            &path_params,
            &dynamic_state,
            LasmDbRecordsAdapter::RecordsLog,
            "rt-unit",
        );

        assert!(
            handled,
            "missing execTx template marker should be handled deterministically"
        );
        assert_eq!(response.status, 400);
        let body = String::from_utf8(response.body).expect("response body should be utf-8 JSON");
        assert!(body.contains("\"code\":\"DB.EXEC_TX_INVALID\""));
    }
}
