use crate::lasm_db_client::{
    ensure_lasm_db_records_client_ready, parse_lasm_db_template_and_params,
    LasmInternalDbSequenceState,
    resolve_lasm_exec_tx_state_bindings_locked, run_lasm_db_tx_allocate_locked_operation,
    run_lasm_exec_operation_with_adapter, run_lasm_exec_tx_operation_with_adapter,
    run_lasm_query_one_operation_with_adapter, LasmExecTxSource, LasmLockedOperationError,
    LasmPreparedDbOperationParams, LasmUnifiedExecOperationError, LasmUnifiedExecTxOperationError,
    LasmUnifiedQueryOneOperationError,
};
use crate::lasm_db_records_response::apply_lasm_db_list_records_response_materialization;
use crate::lasm_db_runtime_common::{
    classify_lasm_db_runtime_error, is_lasm_valid_db_cap_handle,
    normalize_lasm_db_params_and_value, parse_lasm_positive_i64,
};
use crate::{
    lasm_db_record_to_json, lasm_error_envelope, lasm_internal_db_indexed_header, lasm_now_ms,
    set_lasm_json_response, LasmDbRecord, LasmDbRecordsAdapter, LasmDynamicResponseState,
    LasmRunRequest, LASM_INTERNAL_DB_HANDLE_HEADER, LASM_INTERNAL_DB_OP_COUNT_HEADER,
    LASM_INTERNAL_DB_OP_HEADER, LASM_INTERNAL_DB_OP_SEQUENCE_MAX, LASM_INTERNAL_DB_PARAMS_HEADER,
    LASM_INTERNAL_DB_ROW_SCHEMA_HEADER, LASM_INTERNAL_DB_TEMPLATE_HEADER,
    LASM_INTERNAL_DB_TX_DB_HEADER, LASM_INTERNAL_DB_TX_HEADER, LASM_INTERNAL_DB_TX_RESULT_HEADER,
    LASM_INTERNAL_DB_TX_SEQUENCE_RETAIN_HEADER,
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
        materialize_lasm_internal_header_value(raw_template_header, request, path_params, trace_id);
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
    let params =
        materialize_lasm_internal_header_value(raw_params_header, request, path_params, trace_id);
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
    let db_raw =
        materialize_lasm_internal_header_value(raw_db_header, request, path_params, trace_id);
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
    let row_schema_raw = materialize_lasm_internal_header_value(
        raw_row_schema_header,
        request,
        path_params,
        trace_id,
    );
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

fn parse_lasm_record_params_array(record: &LasmDbRecord) -> Option<Vec<serde_json::Value>> {
    serde_json::from_str::<Vec<serde_json::Value>>(record.params.as_str()).ok()
}

fn parse_lasm_record_flat_param_values(record: &LasmDbRecord) -> Option<Vec<&str>> {
    crate::lasm_request_template::parse_lasm_flat_json_array_elements(record.params.as_str())
}

fn lasm_record_param_string(
    index: usize,
    flat_values: Option<&[&str]>,
    fallback_values: Option<&[serde_json::Value]>,
) -> Option<String> {
    if let Some(raw) = flat_values.and_then(|values| values.get(index)) {
        let raw = raw.trim();
        if raw.len() >= 2 && raw.starts_with('"') && raw.ends_with('"') {
            let decoded = serde_json::from_str::<String>(raw).ok()?;
            let trimmed = decoded.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    let fallback_values = fallback_values?;
    fallback_values
        .get(index)
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned)
}

fn lasm_record_param_i64(
    index: usize,
    flat_values: Option<&[&str]>,
    fallback_values: Option<&[serde_json::Value]>,
) -> Option<i64> {
    if let Some(values) = flat_values {
        let raw = values.get(index)?.trim();
        if raw.is_empty() {
            return None;
        }
        if raw.starts_with('"') && raw.ends_with('"') && raw.len() >= 2 {
            let decoded = serde_json::from_str::<String>(raw).ok()?;
            return decoded.trim().parse::<i64>().ok();
        }
        if let Ok(number) = raw.parse::<i64>() {
            return Some(number);
        }
    }
    let fallback_values = fallback_values?;
    let value = fallback_values.get(index)?;
    value.as_i64().or_else(|| {
        value
            .as_str()
            .and_then(|raw| raw.trim().parse::<i64>().ok())
    })
}

fn derive_lasm_exec_like_success_data(
    record: &LasmDbRecord,
    affected_rows: u64,
) -> serde_json::Value {
    let parsed_flat_values = parse_lasm_record_flat_param_values(record);
    let flat_values = parsed_flat_values.as_deref();
    let parsed_fallback_values = parse_lasm_record_params_array(record);
    let fallback_values = parsed_fallback_values.as_deref();
    if record.op == "execTx" {
        if let (Some(comment_id), Some(task_id)) =
            (
                lasm_record_param_string(0, flat_values, fallback_values),
                lasm_record_param_string(1, flat_values, fallback_values),
            )
        {
            return serde_json::json!({
                "taskId": task_id,
                "commentId": comment_id
            });
        }
    }
    if let Some(id) = lasm_record_param_string(0, flat_values, fallback_values) {
        return serde_json::json!({ "id": id });
    }
    serde_json::json!({ "affectedRows": affected_rows })
}

fn derive_lasm_query_one_success_data(
    record: &LasmDbRecord,
    row_object: &serde_json::Value,
) -> serde_json::Value {
    let parsed_flat_values = parse_lasm_record_flat_param_values(record);
    let flat_values = parsed_flat_values.as_deref();
    let parsed_fallback_values = parse_lasm_record_params_array(record);
    let fallback_values = parsed_fallback_values.as_deref();
    if record
        .template
        .contains("order by created_at_ms desc, id desc limit $2 offset $3")
    {
        let mut limit = 20_i64;
        let mut offset = 0_i64;
        if let Some(value) = lasm_record_param_i64(1, flat_values, fallback_values) {
            limit = value;
        }
        if let Some(value) = lasm_record_param_i64(2, flat_values, fallback_values) {
            offset = value;
        }
        return serde_json::json!({
            "items": [row_object.clone()],
            "count": 1,
            "limit": limit,
            "offset": offset
        });
    }
    row_object.clone()
}

fn set_lasm_db_exec_like_success_response(
    response: &mut sec4_core::HttpResponse,
    record: &LasmDbRecord,
    affected_rows: u64,
    request: &LasmRunRequest,
    trace_id: &str,
) {
    let status = if (200..300).contains(&response.status) {
        response.status
    } else {
        200
    };
    let data = derive_lasm_exec_like_success_data(record, affected_rows);
    if request.path.starts_with("/wb/") {
        set_lasm_json_response(
            response,
            status,
            &serde_json::json!({
                "ok": true,
                "status": status,
                "traceId": trace_id,
                "timeMs": lasm_now_ms(),
                "data": data,
                "tx": record.tx,
                "affectedRows": affected_rows,
            }),
        );
        return;
    }
    set_lasm_json_response(
        response,
        status,
        &serde_json::json!({
            "ok": true,
            "status": status,
            "traceId": trace_id,
            "timeMs": lasm_now_ms(),
            "data": data,
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
    request: &LasmRunRequest,
    trace_id: &str,
) {
    let status = if (200..300).contains(&response.status) {
        response.status
    } else {
        200
    };
    let data = derive_lasm_query_one_success_data(record, row_object);
    if request.path.starts_with("/wb/") {
        set_lasm_json_response(
            response,
            status,
            &serde_json::json!({
                "ok": true,
                "status": status,
                "traceId": trace_id,
                "timeMs": lasm_now_ms(),
                "data": data,
                "rowSchema": row_schema,
                "rowObject": row_object,
            }),
        );
        return;
    }
    set_lasm_json_response(
        response,
        status,
        &serde_json::json!({
            "ok": true,
            "status": status,
            "traceId": trace_id,
            "timeMs": lasm_now_ms(),
            "data": data,
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

fn set_lasm_db_adapter_mismatch_response(response: &mut sec4_core::HttpResponse, trace_id: &str) {
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

fn fail_lasm_internal_db_sequence_with_envelope(
    response: &mut sec4_core::HttpResponse,
    dynamic_state: &Mutex<LasmDynamicResponseState>,
    sequence_state: &LasmInternalDbSequenceState,
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
    sequence_state.cleanup(dynamic_state, db_records_adapter, false);
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
        let mut sequence_state = LasmInternalDbSequenceState::new();
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
                        &sequence_state,
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
                        trace_id,
                    );
                    let Some(tx_db_source) = parse_lasm_positive_i64(tx_db_source_raw.trim())
                    else {
                        return fail_lasm_internal_db_sequence_with_envelope(
                            response,
                            dynamic_state,
                            &sequence_state,
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
                            &sequence_state,
                            db_records_adapter,
                            "DB.EXEC_TX_INVALID",
                            "validation",
                            "db.execTx requires db.tx(dbCap) with valid db capability handle",
                            400,
                            trace_id,
                        );
                    }
                    if let Some(existing_tx_handle) =
                        sequence_state.tracked_tx_for_source(tx_db_source)
                    {
                        response.headers.insert(
                            LASM_INTERNAL_DB_TX_HEADER.to_string(),
                            existing_tx_handle.to_string(),
                        );
                        response.headers.insert(
                            LASM_INTERNAL_DB_TX_SEQUENCE_RETAIN_HEADER.to_string(),
                            "1".to_string(),
                        );
                        sequence_state.track_tx_handle(existing_tx_handle);
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
                    let tx_handle_raw = materialize_lasm_internal_header_value(
                        raw_tx_handle,
                        request,
                        path_params,
                        trace_id,
                    );
                    let Some(tx_handle) = parse_lasm_positive_i64(tx_handle_raw.trim()) else {
                        return fail_lasm_internal_db_sequence_with_envelope(
                            response,
                            dynamic_state,
                            &sequence_state,
                            db_records_adapter,
                            "DB.EXEC_TX_INVALID",
                            "validation",
                            "db.execTx requires valid tx handle",
                            400,
                            trace_id,
                        );
                    };
                    sequence_state.track_tx_handle(tx_handle);
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
                    &sequence_state,
                    db_records_adapter,
                    "DB.OPERATION_INVALID",
                    "validation",
                    "missing internal db operation marker",
                    400,
                    trace_id,
                );
            }
            if response.status >= 400 {
                sequence_state.cleanup(dynamic_state, db_records_adapter, false);
                return true;
            }
            if operation == "tx" {
                let Some(raw_db_source) = sequence_tx_db_source_raw else {
                    return fail_lasm_internal_db_sequence_with_envelope(
                        response,
                        dynamic_state,
                        &sequence_state,
                        db_records_adapter,
                        "DB.TX_INTERNAL",
                        "internal",
                        "db.tx runtime did not preserve db handle marker",
                        500,
                        trace_id,
                    );
                };
                let db_source_raw = materialize_lasm_internal_header_value(
                    raw_db_source,
                    request,
                    path_params,
                    trace_id,
                );
                let Some(db_source) = parse_lasm_positive_i64(db_source_raw.trim()) else {
                    return fail_lasm_internal_db_sequence_with_envelope(
                        response,
                        dynamic_state,
                        &sequence_state,
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
                        &sequence_state,
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
                        &sequence_state,
                        db_records_adapter,
                        "DB.TX_INTERNAL",
                        "internal",
                        "db.tx runtime failure",
                        500,
                        trace_id,
                    );
                };
                sequence_state.track_source_tx_handle(db_source, tx_handle);
            }
            if let Some(tx_db_source) = sequence_allocated_tx_source {
                let Some(tx_handle_raw) =
                    take_lasm_internal_header_value(response, LASM_INTERNAL_DB_TX_RESULT_HEADER)
                else {
                    return fail_lasm_internal_db_sequence_with_envelope(
                        response,
                        dynamic_state,
                        &sequence_state,
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
                        &sequence_state,
                        db_records_adapter,
                        "DB.TX_INTERNAL",
                        "internal",
                        "db.tx runtime failure",
                        500,
                        trace_id,
                    );
                };
                sequence_state.track_source_tx_handle(tx_db_source, tx_handle);
            }
        }
        sequence_state.cleanup(dynamic_state, db_records_adapter, true);
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
    let operation =
        materialize_lasm_internal_header_value(raw_operation, request, path_params, trace_id);
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
    let tx = match run_lasm_db_tx_allocate_locked_operation(dynamic_state, db_records_adapter, db) {
        Ok(value) => value,
        Err(LasmLockedOperationError::StateUnavailable) => {
            set_lasm_dynamic_state_unavailable_response(response, trace_id);
            return true;
        }
        Err(LasmLockedOperationError::AdapterMismatch) => {
            set_lasm_db_adapter_mismatch_response(response, trace_id);
            return true;
        }
        Err(LasmLockedOperationError::Capacity { max_handles }) => {
            set_lasm_db_tx_capacity_response(response, max_handles, trace_id);
            return true;
        }
        Err(LasmLockedOperationError::InvalidHandle)
        | Err(LasmLockedOperationError::ConflictInUse)
        | Err(LasmLockedOperationError::Runtime(_)) => {
            set_lasm_db_runtime_error_response(
                response,
                "tx",
                "internal db.tx allocation failure",
                trace_id,
            );
            return true;
        }
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
    let success = match run_lasm_exec_operation_with_adapter(
        dynamic_state,
        db_records_adapter,
        db,
        template.as_str(),
        params.as_str(),
        &prepared_params,
    ) {
        Ok(value) => value,
        Err(LasmUnifiedExecOperationError::StateUnavailable) => {
            set_lasm_dynamic_state_unavailable_response(response, trace_id);
            return true;
        }
        Err(LasmUnifiedExecOperationError::AdapterMismatch) => {
            set_lasm_db_adapter_mismatch_response(response, trace_id);
            return true;
        }
        Err(LasmUnifiedExecOperationError::PreparationMismatch) => {
            set_lasm_db_preparse_mismatch_response(response, "exec", trace_id);
            return true;
        }
        Err(LasmUnifiedExecOperationError::Runtime(message)) => {
            set_lasm_db_runtime_error_response(response, "exec", message.as_str(), trace_id);
            return true;
        }
    };
    set_lasm_db_exec_like_success_response(
        response,
        &success.record,
        success.record.affected_rows,
        request,
        trace_id,
    );
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
        .map(|value| materialize_lasm_internal_header_value(value, request, path_params, trace_id));
    let tx_handle_raw = take_lasm_internal_header_value(response, LASM_INTERNAL_DB_TX_HEADER)
        .map(|value| materialize_lasm_internal_header_value(value, request, path_params, trace_id));
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
    let bindings = match resolve_lasm_exec_tx_state_bindings_locked(
        dynamic_state,
        db_records_adapter,
        &tx_source,
    ) {
        Ok(value) => value,
        Err(LasmLockedOperationError::StateUnavailable) => {
            set_lasm_dynamic_state_unavailable_response(response, trace_id);
            return true;
        }
        Err(LasmLockedOperationError::AdapterMismatch) => {
            set_lasm_db_adapter_mismatch_response(response, trace_id);
            return true;
        }
        Err(LasmLockedOperationError::Capacity { max_handles }) => {
            set_lasm_db_tx_capacity_response(response, max_handles, trace_id);
            return true;
        }
        Err(LasmLockedOperationError::InvalidHandle) => {
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
            return true;
        }
        Err(LasmLockedOperationError::ConflictInUse) => {
            set_lasm_json_response(
                response,
                409,
                &lasm_error_envelope(
                    "DB.EXEC_TX_CONFLICT",
                    "conflict",
                    "db.execTx transaction handle is already in use",
                    409,
                    trace_id,
                ),
            );
            return true;
        }
        Err(LasmLockedOperationError::Runtime(message)) => {
            set_lasm_db_runtime_error_response(response, "execTx", message.as_str(), trace_id);
            return true;
        }
    };
    let (db, tx, tx_active, allocated_tx_handle) = (
        bindings.db,
        bindings.tx,
        bindings.tx_active,
        bindings.allocated_tx_handle,
    );
    let success = match run_lasm_exec_tx_operation_with_adapter(
        dynamic_state,
        db_records_adapter,
        db,
        tx,
        tx_active,
        allocated_tx_handle,
        keep_allocated_tx_handle,
        template.as_str(),
        params.as_str(),
        &prepared_params,
    ) {
        Ok(value) => value,
        Err(LasmUnifiedExecTxOperationError::StateUnavailable) => {
            set_lasm_dynamic_state_unavailable_response(response, trace_id);
            return true;
        }
        Err(LasmUnifiedExecTxOperationError::AdapterMismatch) => {
            set_lasm_db_adapter_mismatch_response(response, trace_id);
            return true;
        }
        Err(LasmUnifiedExecTxOperationError::PreparationMismatch) => {
            set_lasm_db_preparse_mismatch_response(response, "execTx", trace_id);
            return true;
        }
        Err(LasmUnifiedExecTxOperationError::Runtime(message)) => {
            set_lasm_db_runtime_error_response(response, "execTx", message.as_str(), trace_id);
            return true;
        }
    };
    set_lasm_db_exec_like_success_response(
        response,
        &success.record,
        success.record.affected_rows,
        request,
        trace_id,
    );
    response.headers.insert(
        LASM_INTERNAL_DB_TX_RESULT_HEADER.to_string(),
        success.record.tx.to_string(),
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
    let success = match run_lasm_query_one_operation_with_adapter(
        dynamic_state,
        db_records_adapter,
        db,
        template.as_str(),
        params.as_str(),
        row_schema,
        &prepared_params,
    ) {
        Ok(value) => value,
        Err(LasmUnifiedQueryOneOperationError::StateUnavailable) => {
            set_lasm_dynamic_state_unavailable_response(response, trace_id);
            return true;
        }
        Err(LasmUnifiedQueryOneOperationError::AdapterMismatch) => {
            set_lasm_db_adapter_mismatch_response(response, trace_id);
            return true;
        }
        Err(LasmUnifiedQueryOneOperationError::NotFound) => {
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
        Err(LasmUnifiedQueryOneOperationError::PreparationMismatch) => {
            set_lasm_db_preparse_mismatch_response(response, "queryOne", trace_id);
            return true;
        }
        Err(LasmUnifiedQueryOneOperationError::Runtime(message)) => {
            set_lasm_db_runtime_error_response(response, "queryOne", message.as_str(), trace_id);
            return true;
        }
    };
    let row = serde_json::to_string(&success.row_object).unwrap_or_else(|_| "{}".to_string());
    if !enforce_lasm_db_query_one_row_max_columns(response, &success.row_object, trace_id) {
        return true;
    }
    if !enforce_lasm_db_query_one_row_max_bytes(response, row.as_str(), trace_id) {
        return true;
    }
    set_lasm_db_query_one_success_response(
        response,
        &success.record,
        row_schema,
        row.as_str(),
        &success.row_object,
        request,
        trace_id,
    );
    true
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
    trace_id: &str,
) -> String {
    if crate::contains_lasm_request_placeholder_tokens(value.as_str()) {
        crate::materialize_lasm_request_placeholders(value.as_str(), request, path_params, trace_id)
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use super::{
        apply_lasm_internal_db_operation_materialization, derive_lasm_exec_like_success_data,
        derive_lasm_query_one_success_data, lasm_record_param_i64, lasm_record_param_string,
    };
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

    #[test]
    fn exec_tx_sequence_reuses_inline_tx_source_across_multiple_steps() {
        let request = empty_request();
        let path_params = BTreeMap::new();
        let dynamic_state = Mutex::new(LasmDynamicResponseState {
            db_records_max: 8,
            db_tx_max_handles: 8,
            ..LasmDynamicResponseState::default()
        });
        let mut response = sec4_core::HttpResponse::text(200, "ok");
        response.headers.insert(
            LASM_INTERNAL_DB_OP_COUNT_HEADER.to_string(),
            "3".to_string(),
        );
        response.headers.insert(
            lasm_internal_db_indexed_header(LASM_INTERNAL_DB_OP_HEADER, 0),
            "tx".to_string(),
        );
        response.headers.insert(
            lasm_internal_db_indexed_header(LASM_INTERNAL_DB_HANDLE_HEADER, 0),
            "1".to_string(),
        );
        response.headers.insert(
            lasm_internal_db_indexed_header(LASM_INTERNAL_DB_OP_HEADER, 1),
            "execTx".to_string(),
        );
        response.headers.insert(
            lasm_internal_db_indexed_header(LASM_INTERNAL_DB_TEMPLATE_HEADER, 1),
            "INSERT INTO logs VALUES ($1)".to_string(),
        );
        response.headers.insert(
            lasm_internal_db_indexed_header(LASM_INTERNAL_DB_PARAMS_HEADER, 1),
            "[\"alpha\"]".to_string(),
        );
        response.headers.insert(
            lasm_internal_db_indexed_header(LASM_INTERNAL_DB_TX_DB_HEADER, 1),
            "1".to_string(),
        );
        response.headers.insert(
            lasm_internal_db_indexed_header(LASM_INTERNAL_DB_OP_HEADER, 2),
            "execTx".to_string(),
        );
        response.headers.insert(
            lasm_internal_db_indexed_header(LASM_INTERNAL_DB_TEMPLATE_HEADER, 2),
            "INSERT INTO logs VALUES ($1)".to_string(),
        );
        response.headers.insert(
            lasm_internal_db_indexed_header(LASM_INTERNAL_DB_PARAMS_HEADER, 2),
            "[\"beta\"]".to_string(),
        );
        response.headers.insert(
            lasm_internal_db_indexed_header(LASM_INTERNAL_DB_TX_DB_HEADER, 2),
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
            "sequence materialization should handle tx + execTx + execTx"
        );
        assert_eq!(response.status, 200);
        let state = dynamic_state.lock().expect("dynamic state should lock");
        assert_eq!(
            state.db_records.len(),
            2,
            "both execTx calls should append records"
        );
        assert!(
            state.db_tx_handles.is_empty(),
            "sequence cleanup should release tx handles"
        );
    }

    #[test]
    fn derive_exec_like_success_data_extracts_ids_from_flat_params() {
        let tx_record = LasmDbRecord {
            id: 1,
            op: "execTx".to_string(),
            db: 1,
            template: "INSERT INTO wb_comments VALUES ($1,$2)".to_string(),
            params: "[\"comment-1\",\"task-1\"]".to_string(),
            tx: 1,
            affected_rows: 1,
            created_at_ms: 1,
        };
        let tx_data = derive_lasm_exec_like_success_data(&tx_record, 1);
        assert_eq!(tx_data.get("commentId").and_then(|value| value.as_str()), Some("comment-1"));
        assert_eq!(tx_data.get("taskId").and_then(|value| value.as_str()), Some("task-1"));

        let exec_record = LasmDbRecord {
            id: 2,
            op: "exec".to_string(),
            db: 1,
            template: "INSERT INTO wb_tasks VALUES ($1)".to_string(),
            params: "[\"task-9\"]".to_string(),
            tx: 0,
            affected_rows: 1,
            created_at_ms: 2,
        };
        let exec_data = derive_lasm_exec_like_success_data(&exec_record, 1);
        assert_eq!(exec_data.get("id").and_then(|value| value.as_str()), Some("task-9"));
    }

    #[test]
    fn lasm_record_param_string_supports_json_fallback_values() {
        let fallback_values = serde_json::from_str::<Vec<serde_json::Value>>(
            "[\"task-fallback\",{\"x\":1}]",
        )
        .expect("json fallback values should parse");
        let id = lasm_record_param_string(0, None, Some(fallback_values.as_slice()));
        assert_eq!(id.as_deref(), Some("task-fallback"));
    }

    #[test]
    fn derive_query_one_success_data_extracts_limit_offset_from_flat_params() {
        let record = LasmDbRecord {
            id: 3,
            op: "queryOne".to_string(),
            db: 1,
            template: "select id from wb_tasks where status = $1 order by created_at_ms desc, id desc limit $2 offset $3".to_string(),
            params: "[\"open\",20,5]".to_string(),
            tx: 0,
            affected_rows: 0,
            created_at_ms: 3,
        };
        let row_object = serde_json::json!({ "id": "task-1" });
        let data = derive_lasm_query_one_success_data(&record, &row_object);

        assert_eq!(data.get("limit").and_then(|value| value.as_i64()), Some(20));
        assert_eq!(data.get("offset").and_then(|value| value.as_i64()), Some(5));
        let items = data
            .get("items")
            .and_then(|value| value.as_array())
            .expect("query-one list payload should include items array");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].get("id").and_then(|value| value.as_str()), Some("task-1"));
    }

    #[test]
    fn lasm_record_param_i64_supports_json_fallback_values() {
        let fallback_values = serde_json::from_str::<Vec<serde_json::Value>>(
            "[{\"skip\":1},\"25\",\"7\"]",
        )
        .expect("json fallback values should parse");
        let limit = lasm_record_param_i64(1, None, Some(fallback_values.as_slice()));
        let offset = lasm_record_param_i64(2, None, Some(fallback_values.as_slice()));
        assert_eq!(limit, Some(25));
        assert_eq!(offset, Some(7));
    }
}
