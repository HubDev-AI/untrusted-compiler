use crate::lasm_db_client::{
    cleanup_lasm_internal_db_sequence_tx_handles as cleanup_lasm_internal_db_sequence_tx_handles_from_adapter,
    ensure_lasm_db_records_client_ready, parse_lasm_db_template_and_params,
    persist_lasm_db_record_after_unlock, persist_lasm_db_record_append,
    persist_lasm_db_records_full_sync, run_lasm_db_exec_operation, run_lasm_db_exec_tx_operation,
    run_lasm_db_query_one_operation, run_lasm_db_tx_commit, run_lasm_db_tx_rollback,
    LasmDbExecOperationResult, LasmDbExecTxOperationResult, LasmDbQueryOneOperationError,
    LasmDbQueryOneOperationResult, LasmPreparedDbOperationParams,
};
use crate::lasm_db_records_response::apply_lasm_db_list_records_response_materialization;
use crate::lasm_db_runtime_common::{
    allocate_lasm_db_tx_handle, classify_lasm_db_runtime_error, is_lasm_valid_db_cap_handle,
    normalize_lasm_db_params_and_value, parse_lasm_positive_i64,
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
use std::collections::{BTreeMap, BTreeSet};
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
static LASM_DB_OP_SEQUENCE_MAX_OVERRIDE: AtomicUsize = AtomicUsize::new(0);
static LASM_DB_QUERY_ONE_ROW_MAX_BYTES_OVERRIDE: AtomicUsize = AtomicUsize::new(0);
static LASM_DB_QUERY_ONE_ROW_MAX_COLUMNS_OVERRIDE: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct LasmDbRuntimeLimitOverrides {
    pub(crate) query_one_row_max_bytes: Option<usize>,
    pub(crate) query_one_row_max_columns: Option<usize>,
    pub(crate) sql_template_max_bytes: Option<usize>,
    pub(crate) params_max_bytes: Option<usize>,
    pub(crate) params_max_entries: Option<usize>,
    pub(crate) op_sequence_max: Option<usize>,
}

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
    let override_value = LASM_DB_OP_SEQUENCE_MAX_OVERRIDE.load(Ordering::Relaxed);
    if override_value != 0 {
        return override_value.clamp(LASM_DB_OP_SEQUENCE_MAX_MIN, LASM_DB_OP_SEQUENCE_MAX_MAX);
    }
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
pub(crate) fn set_lasm_db_op_sequence_max_override(value: Option<usize>) {
    LASM_DB_OP_SEQUENCE_MAX_OVERRIDE.store(value.unwrap_or(0), Ordering::Relaxed);
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

pub(crate) fn apply_lasm_db_runtime_limit_overrides(overrides: LasmDbRuntimeLimitOverrides) {
    set_lasm_db_query_one_row_max_bytes_override(overrides.query_one_row_max_bytes);
    set_lasm_db_query_one_row_max_columns_override(overrides.query_one_row_max_columns);
    set_lasm_db_sql_template_max_bytes_override(overrides.sql_template_max_bytes);
    set_lasm_db_params_max_bytes_override(overrides.params_max_bytes);
    set_lasm_db_params_max_entries_override(overrides.params_max_entries);
    set_lasm_db_op_sequence_max_override(overrides.op_sequence_max);
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
    if let Err(message) = persist_lasm_db_record_append(state, record) {
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
        if let Err(message) = persist_lasm_db_records_full_sync(state) {
            eprintln!(
                "warning: LASM dynamic records store compaction sync failed after \
                 in-memory overflow: {message}"
            );
        }
    }
}

fn allocate_lasm_db_runtime_record(
    state: &mut LasmDynamicResponseState,
    op: &str,
    db: i64,
    template: &str,
    params: &str,
    tx: i64,
    affected_rows: u64,
) -> LasmDbRecord {
    let record = LasmDbRecord {
        id: state.next_db_record_id,
        op: op.to_string(),
        db,
        template: template.to_string(),
        params: params.to_string(),
        tx,
        affected_rows,
        created_at_ms: lasm_now_ms(),
    };
    state.next_db_record_id = state.next_db_record_id.saturating_add(1);
    record
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

enum LasmExecTxSource {
    AllocateFromDb(i64),
    ExistingTx(i64),
}

fn parse_lasm_db_operation_params(
    db_records_adapter: LasmDbRecordsAdapter,
    validation_code: &'static str,
    template: &str,
    params: &str,
    parsed_params: Option<&serde_json::Value>,
    response: &mut sec4_core::HttpResponse,
    trace_id: &str,
) -> Option<LasmPreparedDbOperationParams> {
    match parse_lasm_db_template_and_params(db_records_adapter, template, params, parsed_params) {
        Ok(params) => Some(params),
        Err(message) => {
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
            None
        }
    }
}

fn prepare_lasm_db_operation_params(
    db_records_adapter: LasmDbRecordsAdapter,
    validation_code: &'static str,
    template: &str,
    params: &str,
    parsed_params: Option<&serde_json::Value>,
    response: &mut sec4_core::HttpResponse,
    trace_id: &str,
) -> Option<LasmPreparedDbOperationParams> {
    parse_lasm_db_operation_params(
        db_records_adapter,
        validation_code,
        template,
        params,
        parsed_params,
        response,
        trace_id,
    )
}

fn resolve_lasm_db_operation_template_and_params(
    response: &mut sec4_core::HttpResponse,
    request: &LasmRunRequest,
    path_params: &BTreeMap<String, String>,
    operation: &str,
    validation_code: &'static str,
    missing_template_message: &'static str,
    missing_params_message: &'static str,
    trace_id: &str,
) -> Option<(String, String, Option<serde_json::Value>)> {
    let Some(raw_template_header) =
        take_lasm_internal_header_value(response, LASM_INTERNAL_DB_TEMPLATE_HEADER)
    else {
        set_lasm_json_response(
            response,
            400,
            &lasm_error_envelope(
                validation_code,
                "validation",
                missing_template_message,
                400,
                trace_id,
            ),
        );
        return None;
    };
    let template =
        materialize_lasm_internal_header_value(raw_template_header, request, path_params);
    if !enforce_lasm_db_sql_template_max_bytes(response, operation, template.as_str(), trace_id) {
        return None;
    }
    if template.trim().is_empty() {
        let code = match operation {
            "exec" => "DB.EXEC_INVALID",
            "execTx" => "DB.EXEC_TX_INVALID",
            "queryOne" => "DB.QUERY_ONE_INVALID",
            _ => "DB.SQL_TEMPLATE_INVALID",
        };
        set_lasm_json_response(
            response,
            400,
            &lasm_error_envelope(
                code,
                "validation",
                "sql.q query template is required",
                400,
                trace_id,
            ),
        );
        return None;
    }
    let Some(raw_params_header) =
        take_lasm_internal_header_value(response, LASM_INTERNAL_DB_PARAMS_HEADER)
    else {
        set_lasm_json_response(
            response,
            400,
            &lasm_error_envelope(
                validation_code,
                "validation",
                missing_params_message,
                400,
                trace_id,
            ),
        );
        return None;
    };
    let params = materialize_lasm_internal_header_value(raw_params_header, request, path_params);
    if !enforce_lasm_db_params_required(response, operation, params.as_str(), trace_id) {
        return None;
    }
    if !enforce_lasm_db_params_max_bytes(response, operation, params.as_str(), trace_id) {
        return None;
    }
    let template = template.trim().to_string();
    let (params, parsed_params) = normalize_lasm_db_params_and_value(params.as_str());
    if !enforce_lasm_db_params_max_entries(
        response,
        operation,
        parsed_params.as_ref(),
        params.as_str(),
        trace_id,
    ) {
        return None;
    }
    Some((template, params, parsed_params))
}

fn resolve_lasm_db_operation_db_cap_handle(
    response: &mut sec4_core::HttpResponse,
    request: &LasmRunRequest,
    path_params: &BTreeMap<String, String>,
    validation_code: &'static str,
    missing_handles_message: &'static str,
    trace_id: &str,
) -> Option<i64> {
    let Some(raw_db_header) =
        take_lasm_internal_header_value(response, LASM_INTERNAL_DB_HANDLE_HEADER)
    else {
        set_lasm_json_response(
            response,
            400,
            &lasm_error_envelope(
                validation_code,
                "validation",
                missing_handles_message,
                400,
                trace_id,
            ),
        );
        return None;
    };
    let db_raw = materialize_lasm_internal_header_value(raw_db_header, request, path_params);
    let Some(db) = parse_lasm_positive_i64(db_raw.trim()) else {
        set_lasm_json_response(
            response,
            400,
            &lasm_error_envelope(
                validation_code,
                "validation",
                missing_handles_message,
                400,
                trace_id,
            ),
        );
        return None;
    };
    if !is_lasm_valid_db_cap_handle(db) {
        set_lasm_json_response(
            response,
            400,
            &lasm_error_envelope(
                validation_code,
                "validation",
                missing_handles_message,
                400,
                trace_id,
            ),
        );
        return None;
    }
    Some(db)
}

fn resolve_lasm_db_operation_row_schema_handle(
    response: &mut sec4_core::HttpResponse,
    request: &LasmRunRequest,
    path_params: &BTreeMap<String, String>,
    validation_code: &'static str,
    missing_handles_message: &'static str,
    trace_id: &str,
) -> Option<i64> {
    let Some(raw_row_schema_header) =
        take_lasm_internal_header_value(response, LASM_INTERNAL_DB_ROW_SCHEMA_HEADER)
    else {
        set_lasm_json_response(
            response,
            400,
            &lasm_error_envelope(
                validation_code,
                "validation",
                missing_handles_message,
                400,
                trace_id,
            ),
        );
        return None;
    };
    let row_schema_raw =
        materialize_lasm_internal_header_value(raw_row_schema_header, request, path_params);
    let Some(row_schema) = parse_lasm_positive_i64(row_schema_raw.trim()) else {
        set_lasm_json_response(
            response,
            400,
            &lasm_error_envelope(
                validation_code,
                "validation",
                missing_handles_message,
                400,
                trace_id,
            ),
        );
        return None;
    };
    Some(row_schema)
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

fn set_lasm_db_exec_like_success_response(
    response: &mut sec4_core::HttpResponse,
    record: &LasmDbRecord,
    affected_rows: u64,
) {
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
}

fn set_lasm_db_tx_success_response(response: &mut sec4_core::HttpResponse, db: i64, tx: i64) {
    set_lasm_json_response(
        response,
        200,
        &serde_json::json!({
            "ok": true,
            "db": db,
            "op": "tx",
            "tx": tx,
        }),
    );
}

fn set_lasm_db_query_one_success_response(
    response: &mut sec4_core::HttpResponse,
    record: &LasmDbRecord,
    row_schema: i64,
    row: &str,
    row_object: &serde_json::Value,
) {
    set_lasm_json_response(
        response,
        200,
        &serde_json::json!({
            "ok": true,
            "recordId": record.id,
            "rowSchema": row_schema,
            "row": row,
            "rowObject": row_object,
            "record": lasm_db_record_to_json(record),
        }),
    );
}

fn set_lasm_db_preparse_mismatch_response(
    response: &mut sec4_core::HttpResponse,
    operation: &str,
    trace_id: &str,
) {
    let code = match operation {
        "exec" => "DB.EXEC_INTERNAL",
        "execTx" => "DB.EXEC_TX_INTERNAL",
        "queryOne" => "DB.QUERY_ONE_INTERNAL",
        _ => "DB.OPERATION_INTERNAL",
    };
    set_lasm_json_response(
        response,
        500,
        &lasm_error_envelope(
            code,
            "internal",
            "internal db operation preparation mismatch",
            500,
            trace_id,
        ),
    );
}

fn set_lasm_db_tx_capacity_response(
    response: &mut sec4_core::HttpResponse,
    max_handles: usize,
    trace_id: &str,
) {
    let message = format!("db.tx handle capacity reached (max {max_handles})");
    set_lasm_json_response(
        response,
        429,
        &lasm_error_envelope(
            "DB.TX_CAPACITY",
            "resource_limit",
            message.as_str(),
            429,
            trace_id,
        ),
    );
}

fn ensure_lasm_db_adapter_state_match(
    response: &mut sec4_core::HttpResponse,
    state: &LasmDynamicResponseState,
    expected: LasmDbRecordsAdapter,
    trace_id: &str,
) -> bool {
    if state.db_records_adapter == expected {
        return true;
    }
    set_lasm_json_response(
        response,
        500,
        &lasm_error_envelope(
            "DB.ADAPTER_MISMATCH",
            "internal",
            "internal db adapter state mismatch",
            500,
            trace_id,
        ),
    );
    false
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
) -> Option<(i64, i64, bool, Option<i64>)> {
    match tx_source {
        LasmExecTxSource::AllocateFromDb(db_value) => {
            let Some(tx_value) = allocate_lasm_db_tx_handle(state, *db_value) else {
                set_lasm_db_tx_capacity_response(response, state.db_tx_max_handles, trace_id);
                return None;
            };
            Some((*db_value, tx_value, false, Some(tx_value)))
        }
        LasmExecTxSource::ExistingTx(tx_value) => {
            let Some(tx_state) = state.db_tx_handles.get(tx_value).copied() else {
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
            Some((tx_state.db, *tx_value, tx_state.active, None))
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

fn clear_lasm_internal_db_materialization_headers(response: &mut sec4_core::HttpResponse) {
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
}

fn stage_lasm_internal_db_sequence_operation_headers(
    response: &mut sec4_core::HttpResponse,
    index: usize,
    raw_operation: String,
) {
    clear_lasm_internal_db_materialization_headers(response);
    response
        .headers
        .insert(LASM_INTERNAL_DB_OP_HEADER.to_string(), raw_operation);
    if let Some(value) =
        take_lasm_internal_header_value_indexed(response, LASM_INTERNAL_DB_HANDLE_HEADER, index)
    {
        response
            .headers
            .insert(LASM_INTERNAL_DB_HANDLE_HEADER.to_string(), value);
    }
    if let Some(value) =
        take_lasm_internal_header_value_indexed(response, LASM_INTERNAL_DB_TEMPLATE_HEADER, index)
    {
        response
            .headers
            .insert(LASM_INTERNAL_DB_TEMPLATE_HEADER.to_string(), value);
    }
    if let Some(value) =
        take_lasm_internal_header_value_indexed(response, LASM_INTERNAL_DB_PARAMS_HEADER, index)
    {
        response
            .headers
            .insert(LASM_INTERNAL_DB_PARAMS_HEADER.to_string(), value);
    }
    if let Some(value) =
        take_lasm_internal_header_value_indexed(response, LASM_INTERNAL_DB_ROW_SCHEMA_HEADER, index)
    {
        response
            .headers
            .insert(LASM_INTERNAL_DB_ROW_SCHEMA_HEADER.to_string(), value);
    }
}

fn cleanup_lasm_internal_db_sequence_tx_handles_for_sources(
    dynamic_state: &Mutex<LasmDynamicResponseState>,
    sequence_tx_handles_by_source: &BTreeMap<i64, i64>,
    sequence_tx_handles: &BTreeSet<i64>,
    db_records_adapter: LasmDbRecordsAdapter,
    operation_succeeded: bool,
) {
    cleanup_lasm_internal_db_sequence_tx_handles(
        dynamic_state,
        db_records_adapter,
        sequence_tx_handles_by_source.values().copied(),
        operation_succeeded,
    );
    cleanup_lasm_internal_db_sequence_tx_handles(
        dynamic_state,
        db_records_adapter,
        sequence_tx_handles.iter().copied(),
        operation_succeeded,
    );
}

fn fail_lasm_internal_db_sequence_with_envelope(
    response: &mut sec4_core::HttpResponse,
    dynamic_state: &Mutex<LasmDynamicResponseState>,
    sequence_tx_handles_by_source: &BTreeMap<i64, i64>,
    sequence_tx_handles: &BTreeSet<i64>,
    db_records_adapter: LasmDbRecordsAdapter,
    code: &str,
    kind: &str,
    message: &str,
    status: u16,
    trace_id: &str,
) -> bool {
    set_lasm_json_response(
        response,
        status,
        &lasm_error_envelope(code, kind, message, status, trace_id),
    );
    cleanup_lasm_internal_db_sequence_tx_handles_for_sources(
        dynamic_state,
        sequence_tx_handles_by_source,
        sequence_tx_handles,
        db_records_adapter,
        false,
    );
    true
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
        let mut sequence_tx_handles = BTreeSet::<i64>::new();
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

            stage_lasm_internal_db_sequence_operation_headers(response, index, raw_operation);
            let sequence_tx_db_source_raw = if operation == "tx" {
                response
                    .headers
                    .get(LASM_INTERNAL_DB_HANDLE_HEADER)
                    .cloned()
            } else {
                None
            };
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
                    return fail_lasm_internal_db_sequence_with_envelope(
                        response,
                        dynamic_state,
                        &sequence_tx_handles_by_source,
                        &sequence_tx_handles,
                        db_records_adapter,
                        "DB.EXEC_TX_INVALID",
                        "validation",
                        "db.execTx must include either tx handle or db.tx(dbCap) source, not both",
                        400,
                        trace_id,
                    );
                }
                if let Some(raw_tx_db) = raw_tx_db {
                    let tx_db_source_raw = materialize_lasm_internal_header_value(
                        raw_tx_db.clone(),
                        request,
                        path_params,
                    );
                    let Some(tx_db_source) = parse_lasm_positive_i64(tx_db_source_raw.trim())
                    else {
                        return fail_lasm_internal_db_sequence_with_envelope(
                            response,
                            dynamic_state,
                            &sequence_tx_handles_by_source,
                            &sequence_tx_handles,
                            db_records_adapter,
                            "DB.EXEC_TX_INVALID",
                            "validation",
                            "db.execTx requires transaction and query handles",
                            400,
                            trace_id,
                        );
                    };
                    if !is_lasm_valid_db_cap_handle(tx_db_source) {
                        return fail_lasm_internal_db_sequence_with_envelope(
                            response,
                            dynamic_state,
                            &sequence_tx_handles_by_source,
                            &sequence_tx_handles,
                            db_records_adapter,
                            "DB.EXEC_TX_INVALID",
                            "validation",
                            "db.execTx requires db.tx(dbCap) with valid db capability handle",
                            400,
                            trace_id,
                        );
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
                    response.headers.insert(
                        LASM_INTERNAL_DB_TX_SEQUENCE_RETAIN_HEADER.to_string(),
                        "1".to_string(),
                    );
                    response.headers.insert(
                        LASM_INTERNAL_DB_TX_HEADER.to_string(),
                        raw_tx_handle.clone(),
                    );
                    let tx_handle_raw =
                        materialize_lasm_internal_header_value(raw_tx_handle, request, path_params);
                    let Some(tx_handle) = parse_lasm_positive_i64(tx_handle_raw.trim()) else {
                        return fail_lasm_internal_db_sequence_with_envelope(
                            response,
                            dynamic_state,
                            &sequence_tx_handles_by_source,
                            &sequence_tx_handles,
                            db_records_adapter,
                            "DB.EXEC_TX_INVALID",
                            "validation",
                            "db.execTx requires valid tx handle",
                            400,
                            trace_id,
                        );
                    };
                    sequence_tx_handles.insert(tx_handle);
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

            if !apply_lasm_internal_db_operation_materialization_single(
                response,
                request,
                path_params,
                dynamic_state,
                db_records_adapter,
                trace_id,
            ) {
                return fail_lasm_internal_db_sequence_with_envelope(
                    response,
                    dynamic_state,
                    &sequence_tx_handles_by_source,
                    &sequence_tx_handles,
                    db_records_adapter,
                    "DB.OPERATION_INVALID",
                    "validation",
                    "missing internal db operation marker",
                    400,
                    trace_id,
                );
            }
            if response.status >= 400 {
                cleanup_lasm_internal_db_sequence_tx_handles_for_sources(
                    dynamic_state,
                    &sequence_tx_handles_by_source,
                    &sequence_tx_handles,
                    db_records_adapter,
                    false,
                );
                return true;
            }
            if operation == "tx" {
                let Some(raw_db_source) = sequence_tx_db_source_raw else {
                    return fail_lasm_internal_db_sequence_with_envelope(
                        response,
                        dynamic_state,
                        &sequence_tx_handles_by_source,
                        &sequence_tx_handles,
                        db_records_adapter,
                        "DB.TX_INTERNAL",
                        "internal",
                        "db.tx runtime did not preserve db handle marker",
                        500,
                        trace_id,
                    );
                };
                let db_source_raw =
                    materialize_lasm_internal_header_value(raw_db_source, request, path_params);
                let Some(db_source) = parse_lasm_positive_i64(db_source_raw.trim()) else {
                    return fail_lasm_internal_db_sequence_with_envelope(
                        response,
                        dynamic_state,
                        &sequence_tx_handles_by_source,
                        &sequence_tx_handles,
                        db_records_adapter,
                        "DB.TX_INTERNAL",
                        "internal",
                        "db.tx runtime failure",
                        500,
                        trace_id,
                    );
                };
                let Some(tx_handle_raw) =
                    take_lasm_internal_header_value(response, LASM_INTERNAL_DB_TX_RESULT_HEADER)
                else {
                    return fail_lasm_internal_db_sequence_with_envelope(
                        response,
                        dynamic_state,
                        &sequence_tx_handles_by_source,
                        &sequence_tx_handles,
                        db_records_adapter,
                        "DB.TX_INTERNAL",
                        "internal",
                        "db.tx runtime did not publish transaction handle marker",
                        500,
                        trace_id,
                    );
                };
                let Some(tx_handle) = parse_lasm_positive_i64(tx_handle_raw.as_str()) else {
                    return fail_lasm_internal_db_sequence_with_envelope(
                        response,
                        dynamic_state,
                        &sequence_tx_handles_by_source,
                        &sequence_tx_handles,
                        db_records_adapter,
                        "DB.TX_INTERNAL",
                        "internal",
                        "db.tx runtime failure",
                        500,
                        trace_id,
                    );
                };
                sequence_tx_handles_by_source.insert(db_source, tx_handle);
            }
            if let Some(tx_db_source) = sequence_allocated_tx_source {
                let Some(tx_handle_raw) =
                    take_lasm_internal_header_value(response, LASM_INTERNAL_DB_TX_RESULT_HEADER)
                else {
                    return fail_lasm_internal_db_sequence_with_envelope(
                        response,
                        dynamic_state,
                        &sequence_tx_handles_by_source,
                        &sequence_tx_handles,
                        db_records_adapter,
                        "DB.TX_INTERNAL",
                        "internal",
                        "db.execTx runtime did not publish transaction handle marker",
                        500,
                        trace_id,
                    );
                };
                let Some(tx_handle) = parse_lasm_positive_i64(tx_handle_raw.as_str()) else {
                    return fail_lasm_internal_db_sequence_with_envelope(
                        response,
                        dynamic_state,
                        &sequence_tx_handles_by_source,
                        &sequence_tx_handles,
                        db_records_adapter,
                        "DB.TX_INTERNAL",
                        "internal",
                        "db.tx runtime failure",
                        500,
                        trace_id,
                    );
                };
                sequence_tx_handles_by_source.insert(tx_db_source, tx_handle);
            }
        }
        cleanup_lasm_internal_db_sequence_tx_handles_for_sources(
            dynamic_state,
            &sequence_tx_handles_by_source,
            &sequence_tx_handles,
            db_records_adapter,
            true,
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
            {
                let mut state =
                    match lock_lasm_dynamic_state_or_respond(dynamic_state, response, trace_id) {
                        Some(state) => state,
                        None => return true,
                    };
                if let Err(message) =
                    ensure_lasm_db_records_client_ready(&mut state, db_records_adapter)
                {
                    set_lasm_db_runtime_error_response(response, "listRecords", &message, trace_id);
                    return true;
                }
            }
            apply_lasm_db_list_records_response_materialization(
                response,
                request,
                dynamic_state,
                trace_id,
            );
            true
        }
        "tx" => handle_lasm_internal_db_tx_operation(
            response,
            request,
            path_params,
            dynamic_state,
            db_records_adapter,
            trace_id,
        ),
        "exec" => handle_lasm_internal_db_exec_operation(
            response,
            request,
            path_params,
            dynamic_state,
            db_records_adapter,
            trace_id,
        ),
        "execTx" => handle_lasm_internal_db_exec_tx_operation(
            response,
            request,
            path_params,
            dynamic_state,
            db_records_adapter,
            trace_id,
        ),
        "queryOne" => handle_lasm_internal_db_query_one_operation(
            response,
            request,
            path_params,
            dynamic_state,
            db_records_adapter,
            trace_id,
        ),
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

fn handle_lasm_internal_db_tx_operation(
    response: &mut sec4_core::HttpResponse,
    request: &LasmRunRequest,
    path_params: &BTreeMap<String, String>,
    dynamic_state: &Mutex<LasmDynamicResponseState>,
    db_records_adapter: LasmDbRecordsAdapter,
    trace_id: &str,
) -> bool {
    let Some(db) = resolve_lasm_db_operation_db_cap_handle(
        response,
        request,
        path_params,
        "DB.TX_INVALID",
        "db.tx requires db capability handle",
        trace_id,
    ) else {
        return true;
    };
    let tx = {
        let mut state = match lock_lasm_dynamic_state_or_respond(dynamic_state, response, trace_id)
        {
            Some(state) => state,
            None => return true,
        };
        if !ensure_lasm_db_adapter_state_match(response, &state, db_records_adapter, trace_id) {
            return true;
        }
        let Some(tx) = allocate_lasm_db_tx_handle(&mut state, db) else {
            set_lasm_db_tx_capacity_response(response, state.db_tx_max_handles, trace_id);
            return true;
        };
        tx
    };
    set_lasm_db_tx_success_response(response, db, tx);
    response.headers.insert(
        LASM_INTERNAL_DB_TX_RESULT_HEADER.to_string(),
        tx.to_string(),
    );
    true
}

fn handle_lasm_internal_db_exec_operation(
    response: &mut sec4_core::HttpResponse,
    request: &LasmRunRequest,
    path_params: &BTreeMap<String, String>,
    dynamic_state: &Mutex<LasmDynamicResponseState>,
    db_records_adapter: LasmDbRecordsAdapter,
    trace_id: &str,
) -> bool {
    let Some((template, params, parsed_params)) = resolve_lasm_db_operation_template_and_params(
        response,
        request,
        path_params,
        "exec",
        "DB.EXEC_INVALID",
        "db.exec requires db capability, query template, and query params",
        "db.exec requires db capability, query template, and query params",
        trace_id,
    ) else {
        return true;
    };
    let Some(db) = resolve_lasm_db_operation_db_cap_handle(
        response,
        request,
        path_params,
        "DB.EXEC_INVALID",
        "db.exec requires db capability handle",
        trace_id,
    ) else {
        return true;
    };
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
    let (record, persistence_payload) = {
        let mut state = match lock_lasm_dynamic_state_or_respond(dynamic_state, response, trace_id)
        {
            Some(state) => state,
            None => return true,
        };
        if !ensure_lasm_db_adapter_state_match(response, &state, db_records_adapter, trace_id) {
            return true;
        }
        let operation_result = run_lasm_db_exec_operation(
            &mut state,
            db_records_adapter,
            template.as_str(),
            &prepared_params,
        )
        .map_err(|message| {
            set_lasm_db_runtime_error_response(response, "exec", message.as_str(), trace_id)
        });
        let operation_result = match operation_result {
            Ok(value) => value,
            Err(()) => return true,
        };
        match operation_result {
            LasmDbExecOperationResult::Postgres {
                config,
                affected_rows,
            } => {
                let record = allocate_lasm_db_runtime_record(
                    &mut state,
                    "exec",
                    db,
                    template.as_str(),
                    params.as_str(),
                    0,
                    affected_rows,
                );
                let (record, compaction_snapshot) =
                    append_lasm_db_record_in_memory_with_compaction_snapshot(&mut state, record);
                (record, Some((config, compaction_snapshot)))
            }
            LasmDbExecOperationResult::Sqlite { affected_rows }
            | LasmDbExecOperationResult::RecordsLog { affected_rows } => {
                let record = allocate_lasm_db_runtime_record(
                    &mut state,
                    "exec",
                    db,
                    template.as_str(),
                    params.as_str(),
                    0,
                    affected_rows,
                );
                persist_lasm_db_record_with_capacity_guard(&mut state, &record);
                (record, None)
            }
        }
    };
    if let Some((postgres_config, compaction_snapshot)) = persistence_payload {
        persist_lasm_db_record_after_unlock(
            db_records_adapter,
            &postgres_config,
            &record,
            compaction_snapshot,
        );
    }
    set_lasm_db_exec_like_success_response(response, &record, record.affected_rows);
    true
}

fn handle_lasm_internal_db_exec_tx_operation(
    response: &mut sec4_core::HttpResponse,
    request: &LasmRunRequest,
    path_params: &BTreeMap<String, String>,
    dynamic_state: &Mutex<LasmDynamicResponseState>,
    db_records_adapter: LasmDbRecordsAdapter,
    trace_id: &str,
) -> bool {
    let Some((template, params, parsed_params)) = resolve_lasm_db_operation_template_and_params(
        response,
        request,
        path_params,
        "execTx",
        "DB.EXEC_TX_INVALID",
        "db.execTx requires transaction, query template, and query params",
        "db.execTx requires transaction, query template, and query params",
        trace_id,
    ) else {
        return true;
    };
    let keep_allocated_tx_handle =
        take_lasm_internal_header_value(response, LASM_INTERNAL_DB_TX_SEQUENCE_RETAIN_HEADER)
            .map(|value| value.trim() == "1")
            .unwrap_or(false);
    let tx_db_source = take_lasm_internal_header_value(response, LASM_INTERNAL_DB_TX_DB_HEADER)
        .map(|value| materialize_lasm_internal_header_value(value, request, path_params));
    let tx_handle_raw = take_lasm_internal_header_value(response, LASM_INTERNAL_DB_TX_HEADER)
        .map(|value| materialize_lasm_internal_header_value(value, request, path_params));
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
    let (record, persistence_payload) = {
        let mut state = match lock_lasm_dynamic_state_or_respond(dynamic_state, response, trace_id)
        {
            Some(state) => state,
            None => return true,
        };
        if !ensure_lasm_db_adapter_state_match(response, &state, db_records_adapter, trace_id) {
            return true;
        }
        let (db, tx, tx_active, allocated_tx_handle) =
            match resolve_lasm_exec_tx_state_bindings(&mut state, &tx_source, response, trace_id) {
                Some(value) => value,
                None => return true,
            };
        let operation_result = match run_lasm_db_exec_tx_operation(
            &mut state,
            db_records_adapter,
            tx,
            tx_active,
            template.as_str(),
            &prepared_params,
        ) {
            Ok(value) => value,
            Err(error) => {
                if error.tx_started {
                    let _ = run_lasm_db_tx_rollback(&mut state, db_records_adapter, tx);
                    if !keep_allocated_tx_handle {
                        state.db_tx_handles.remove(&tx);
                    }
                }
                if let Some(tx_handle) = allocated_tx_handle {
                    if !keep_allocated_tx_handle {
                        state.db_tx_handles.remove(&tx_handle);
                    }
                }
                set_lasm_db_runtime_error_response(
                    response,
                    "execTx",
                    error.message.as_str(),
                    trace_id,
                );
                return true;
            }
        };
        match operation_result {
            LasmDbExecTxOperationResult::Postgres {
                config,
                affected_rows,
                tx_started,
            } => {
                if tx_started && !keep_allocated_tx_handle {
                    if let Err(message) = run_lasm_db_tx_commit(&mut state, db_records_adapter, tx)
                    {
                        state.db_tx_handles.remove(&tx);
                        set_lasm_db_runtime_error_response(
                            response,
                            "execTx",
                            message.as_str(),
                            trace_id,
                        );
                        return true;
                    }
                    state.db_tx_handles.remove(&tx);
                }
                if let Some(tx_handle) = allocated_tx_handle {
                    if !keep_allocated_tx_handle {
                        state.db_tx_handles.remove(&tx_handle);
                    }
                }
                let record = allocate_lasm_db_runtime_record(
                    &mut state,
                    "execTx",
                    db,
                    template.as_str(),
                    params.as_str(),
                    tx,
                    affected_rows,
                );
                let (record, compaction_snapshot) =
                    append_lasm_db_record_in_memory_with_compaction_snapshot(&mut state, record);
                (record, Some((config, compaction_snapshot)))
            }
            other_result => {
                let (affected_rows, tx_started, should_commit_tx) = match other_result {
                    LasmDbExecTxOperationResult::Sqlite {
                        affected_rows,
                        tx_started,
                    } => (affected_rows, tx_started, true),
                    LasmDbExecTxOperationResult::RecordsLog { affected_rows } => {
                        (affected_rows, false, false)
                    }
                    LasmDbExecTxOperationResult::Postgres { .. } => unreachable!(),
                };
                if should_commit_tx && tx_started && !keep_allocated_tx_handle {
                    if let Err(message) = run_lasm_db_tx_commit(&mut state, db_records_adapter, tx)
                    {
                        state.db_tx_handles.remove(&tx);
                        set_lasm_db_runtime_error_response(
                            response,
                            "execTx",
                            message.as_str(),
                            trace_id,
                        );
                        return true;
                    }
                    state.db_tx_handles.remove(&tx);
                }
                if let Some(tx_handle) = allocated_tx_handle {
                    if !keep_allocated_tx_handle {
                        state.db_tx_handles.remove(&tx_handle);
                    }
                }
                let record = allocate_lasm_db_runtime_record(
                    &mut state,
                    "execTx",
                    db,
                    template.as_str(),
                    params.as_str(),
                    tx,
                    affected_rows,
                );
                persist_lasm_db_record_with_capacity_guard(&mut state, &record);
                (record, None)
            }
        }
    };
    if let Some((postgres_config, compaction_snapshot)) = persistence_payload {
        persist_lasm_db_record_after_unlock(
            db_records_adapter,
            &postgres_config,
            &record,
            compaction_snapshot,
        );
    }
    set_lasm_db_exec_like_success_response(response, &record, record.affected_rows);
    response.headers.insert(
        LASM_INTERNAL_DB_TX_RESULT_HEADER.to_string(),
        record.tx.to_string(),
    );
    true
}

fn handle_lasm_internal_db_query_one_operation(
    response: &mut sec4_core::HttpResponse,
    request: &LasmRunRequest,
    path_params: &BTreeMap<String, String>,
    dynamic_state: &Mutex<LasmDynamicResponseState>,
    db_records_adapter: LasmDbRecordsAdapter,
    trace_id: &str,
) -> bool {
    let Some((template, params, parsed_params)) = resolve_lasm_db_operation_template_and_params(
        response,
        request,
        path_params,
        "queryOne",
        "DB.QUERY_ONE_INVALID",
        "db.queryOne requires db capability, query template, query params, and row schema",
        "db.queryOne requires db capability, query template, query params, and row schema",
        trace_id,
    ) else {
        return true;
    };
    let Some(db) = resolve_lasm_db_operation_db_cap_handle(
        response,
        request,
        path_params,
        "DB.QUERY_ONE_INVALID",
        "db.queryOne requires db capability, query template, query params, and row schema handles",
        trace_id,
    ) else {
        return true;
    };
    let Some(row_schema) = resolve_lasm_db_operation_row_schema_handle(
        response,
        request,
        path_params,
        "DB.QUERY_ONE_INVALID",
        "db.queryOne requires db capability, query template, query params, and row schema",
        trace_id,
    ) else {
        return true;
    };
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
    let (query_result, persistence_payload) = {
        let mut state = match lock_lasm_dynamic_state_or_respond(dynamic_state, response, trace_id)
        {
            Some(state) => state,
            None => return true,
        };
        if !ensure_lasm_db_adapter_state_match(response, &state, db_records_adapter, trace_id) {
            return true;
        }
        match run_lasm_db_query_one_operation(
            &mut state,
            db_records_adapter,
            db,
            template.as_str(),
            params.as_str(),
            row_schema,
            &prepared_params,
        ) {
            Ok(LasmDbQueryOneOperationResult::Postgres { config, row }) => {
                let row_object = row;
                let record = allocate_lasm_db_runtime_record(
                    &mut state,
                    "queryOne",
                    db,
                    template.as_str(),
                    params.as_str(),
                    0,
                    1,
                );
                let (record, compaction_snapshot) =
                    append_lasm_db_record_in_memory_with_compaction_snapshot(&mut state, record);
                (
                    Some((record.clone(), row_object)),
                    Some((config, record, compaction_snapshot)),
                )
            }
            Ok(LasmDbQueryOneOperationResult::Sqlite { row })
            | Ok(LasmDbQueryOneOperationResult::RecordsLog { row }) => {
                let record = allocate_lasm_db_runtime_record(
                    &mut state,
                    "queryOne",
                    db,
                    template.as_str(),
                    params.as_str(),
                    0,
                    1,
                );
                persist_lasm_db_record_with_capacity_guard(&mut state, &record);
                (Some((record, row)), None)
            }
            Err(LasmDbQueryOneOperationError::NotFound) => (None, None),
            Err(LasmDbQueryOneOperationError::Runtime(message)) => {
                set_lasm_db_runtime_error_response(
                    response,
                    "queryOne",
                    message.as_str(),
                    trace_id,
                );
                return true;
            }
            Err(LasmDbQueryOneOperationError::PreparationMismatch) => {
                set_lasm_db_preparse_mismatch_response(response, "queryOne", trace_id);
                return true;
            }
        }
    };
    if let Some((postgres_config, record, compaction_snapshot)) = persistence_payload {
        persist_lasm_db_record_after_unlock(
            db_records_adapter,
            &postgres_config,
            &record,
            compaction_snapshot,
        );
    }
    let Some((record, row_object)) = query_result else {
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
    };
    let row = serde_json::to_string(&row_object).unwrap_or_else(|_| "{}".to_string());
    if !enforce_lasm_db_query_one_row_max_columns(response, &row_object, trace_id) {
        return true;
    }
    if !enforce_lasm_db_query_one_row_max_bytes(response, row.as_str(), trace_id) {
        return true;
    }
    set_lasm_db_query_one_success_response(
        response,
        &record,
        row_schema,
        row.as_str(),
        &row_object,
    );
    true
}

fn cleanup_lasm_internal_db_sequence_tx_handles(
    dynamic_state: &Mutex<LasmDynamicResponseState>,
    db_records_adapter: LasmDbRecordsAdapter,
    handles: impl IntoIterator<Item = i64>,
    operation_succeeded: bool,
) {
    cleanup_lasm_internal_db_sequence_tx_handles_from_adapter(
        dynamic_state,
        db_records_adapter,
        handles,
        operation_succeeded,
    );
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
    fn rejects_internal_db_operation_when_runtime_adapter_state_mismatches_dispatch_adapter() {
        let request = empty_request();
        let path_params = BTreeMap::new();
        let dynamic_state = Mutex::new(LasmDynamicResponseState {
            db_records_adapter: LasmDbRecordsAdapter::Sqlite,
            ..LasmDynamicResponseState::default()
        });
        let mut response = sec4_core::HttpResponse::text(200, "");
        response
            .headers
            .insert(LASM_INTERNAL_DB_OP_HEADER.to_string(), "exec".to_string());
        response
            .headers
            .insert(LASM_INTERNAL_DB_HANDLE_HEADER.to_string(), "1".to_string());
        response.headers.insert(
            LASM_INTERNAL_DB_TEMPLATE_HEADER.to_string(),
            "select 1".to_string(),
        );
        response
            .headers
            .insert(LASM_INTERNAL_DB_PARAMS_HEADER.to_string(), "0".to_string());

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
            "adapter mismatch should be handled deterministically"
        );
        assert_eq!(response.status, 500);
        let body = String::from_utf8(response.body).expect("response body should be utf-8 JSON");
        assert!(body.contains("\"code\":\"DB.ADAPTER_MISMATCH\""));
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
    fn exec_marker_rejects_empty_template_header() {
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
            "   ".to_string(),
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
            "empty template marker should be handled deterministically"
        );
        assert_eq!(response.status, 400);
        let body = String::from_utf8(response.body).expect("response body should be utf-8 JSON");
        assert!(body.contains("\"code\":\"DB.EXEC_INVALID\""));
    }

    #[test]
    fn exec_marker_rejects_invalid_db_handle_header() {
        let request = empty_request();
        let path_params = BTreeMap::new();
        let dynamic_state = Mutex::new(LasmDynamicResponseState::default());
        let mut response = sec4_core::HttpResponse::text(200, "");
        response
            .headers
            .insert(LASM_INTERNAL_DB_OP_HEADER.to_string(), "exec".to_string());
        response
            .headers
            .insert(LASM_INTERNAL_DB_HANDLE_HEADER.to_string(), "0".to_string());
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
            "invalid db handle should be handled deterministically"
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
    fn query_one_marker_rejects_invalid_row_schema_header() {
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
            "0".to_string(),
        );
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
            "invalid queryOne row schema should be handled deterministically"
        );
        assert_eq!(response.status, 400);
        let body = String::from_utf8(response.body).expect("response body should be utf-8 JSON");
        assert!(body.contains("\"code\":\"DB.QUERY_ONE_INVALID\""));
    }

    #[test]
    fn query_one_marker_rejects_non_numeric_row_schema_header() {
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
            "invalid".to_string(),
        );
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
            "non-numeric queryOne row schema should be handled deterministically"
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
    fn query_one_marker_rejects_empty_template_header() {
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
            "   ".to_string(),
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
            "empty queryOne template marker should be handled deterministically"
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
    fn exec_tx_marker_rejects_empty_template_header() {
        let request = empty_request();
        let path_params = BTreeMap::new();
        let dynamic_state = Mutex::new(LasmDynamicResponseState::default());
        let mut response = sec4_core::HttpResponse::text(200, "");
        response
            .headers
            .insert(LASM_INTERNAL_DB_OP_HEADER.to_string(), "execTx".to_string());
        response
            .headers
            .insert(LASM_INTERNAL_DB_TX_DB_HEADER.to_string(), "1".to_string());
        response
            .headers
            .insert(LASM_INTERNAL_DB_TEMPLATE_HEADER.to_string(), "".to_string());
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
            "empty template marker should be handled deterministically"
        );
        assert_eq!(response.status, 400);
        let body = String::from_utf8(response.body).expect("response body should be utf-8 JSON");
        assert!(body.contains("\"code\":\"DB.EXEC_TX_INVALID\""));
    }

    #[test]
    fn exec_tx_marker_rejects_invalid_tx_db_handle_header() {
        let request = empty_request();
        let path_params = BTreeMap::new();
        let dynamic_state = Mutex::new(LasmDynamicResponseState::default());
        let mut response = sec4_core::HttpResponse::text(200, "");
        response
            .headers
            .insert(LASM_INTERNAL_DB_OP_HEADER.to_string(), "execTx".to_string());
        response
            .headers
            .insert(LASM_INTERNAL_DB_TX_DB_HEADER.to_string(), "0".to_string());
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
            "invalid tx db handle should be handled deterministically"
        );
        assert_eq!(response.status, 400);
        let body = String::from_utf8(response.body).expect("response body should be utf-8 JSON");
        assert!(body.contains("\"code\":\"DB.EXEC_TX_INVALID\""));
    }

    #[test]
    fn exec_tx_marker_rejects_invalid_tx_handle_header() {
        let request = empty_request();
        let path_params = BTreeMap::new();
        let dynamic_state = Mutex::new(LasmDynamicResponseState::default());
        let mut response = sec4_core::HttpResponse::text(200, "");
        response
            .headers
            .insert(LASM_INTERNAL_DB_OP_HEADER.to_string(), "execTx".to_string());
        response
            .headers
            .insert(LASM_INTERNAL_DB_TX_HEADER.to_string(), "0".to_string());
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
            "invalid tx handle should be handled deterministically"
        );
        assert_eq!(response.status, 400);
        let body = String::from_utf8(response.body).expect("response body should be utf-8 JSON");
        assert!(body.contains("\"code\":\"DB.EXEC_TX_INVALID\""));
        assert!(body.contains("db.execTx requires transaction and query handles"));
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

    #[test]
    fn tx_marker_rejects_when_tx_handle_capacity_is_exhausted() {
        let request = empty_request();
        let path_params = BTreeMap::new();
        let dynamic_state = Mutex::new(LasmDynamicResponseState {
            db_tx_max_handles: 0,
            ..LasmDynamicResponseState::default()
        });
        let mut response = sec4_core::HttpResponse::text(200, "");
        response
            .headers
            .insert(LASM_INTERNAL_DB_OP_HEADER.to_string(), "tx".to_string());
        response
            .headers
            .insert(LASM_INTERNAL_DB_HANDLE_HEADER.to_string(), "1".to_string());

        let handled = apply_lasm_internal_db_operation_materialization(
            &mut response,
            &request,
            &path_params,
            &dynamic_state,
            LasmDbRecordsAdapter::RecordsLog,
            "rt-unit",
        );

        assert!(handled, "tx marker should be handled deterministically");
        assert_eq!(response.status, 429);
        let body = String::from_utf8(response.body).expect("response body should be utf-8 JSON");
        assert!(body.contains("\"code\":\"DB.TX_CAPACITY\""));
        assert!(body.contains("db.tx handle capacity reached (max 0)"));
    }

    #[test]
    fn exec_tx_marker_rejects_when_inline_tx_allocation_capacity_is_exhausted() {
        let request = empty_request();
        let path_params = BTreeMap::new();
        let dynamic_state = Mutex::new(LasmDynamicResponseState {
            db_tx_max_handles: 0,
            ..LasmDynamicResponseState::default()
        });
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
            "inline tx allocation should fail deterministically when capacity is exhausted"
        );
        assert_eq!(response.status, 429);
        let body = String::from_utf8(response.body).expect("response body should be utf-8 JSON");
        assert!(body.contains("\"code\":\"DB.TX_CAPACITY\""));
        assert!(body.contains("db.tx handle capacity reached (max 0)"));
    }
}
