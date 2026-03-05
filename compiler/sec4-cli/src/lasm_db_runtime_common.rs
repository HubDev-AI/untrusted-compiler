use crate::lasm_db_adapter_state::{
    connect_lasm_dynamic_db_records_postgres, ensure_lasm_dynamic_db_records_postgres_schema,
    load_lasm_dynamic_db_records_from_postgres,
};
use crate::lasm_db_config::LASM_DB_POSTGRES_DSN_CONFIG_ERROR_MESSAGE;
use crate::lasm_dynamic_state::{LasmDbRecord, LasmDbTxState};
use crate::LasmDynamicResponseState;
use postgres::{Client as PostgresClient, Statement as PostgresStatement};
use std::collections::{BTreeMap, HashMap, VecDeque};

fn lasm_db_operation_timeout_code(operation: &str) -> &'static str {
    match operation {
        "exec" => "DB.EXEC_TIMEOUT",
        "execTx" => "DB.EXEC_TX_TIMEOUT",
        "queryOne" => "DB.QUERY_ONE_TIMEOUT",
        _ => "DB.OPERATION_TIMEOUT",
    }
}

fn lasm_db_operation_lock_conflict_code(operation: &str) -> &'static str {
    match operation {
        "exec" => "DB.EXEC_LOCK_TIMEOUT",
        "execTx" => "DB.EXEC_TX_LOCK_TIMEOUT",
        "queryOne" => "DB.QUERY_ONE_LOCK_TIMEOUT",
        _ => "DB.OPERATION_LOCK_TIMEOUT",
    }
}

fn lasm_db_operation_validation_code(operation: &str) -> &'static str {
    match operation {
        "exec" => "DB.EXEC_INVALID",
        "execTx" => "DB.EXEC_TX_INVALID",
        "queryOne" => "DB.QUERY_ONE_INVALID",
        _ => "DB.OPERATION_INVALID",
    }
}

fn lasm_db_operation_conflict_code(operation: &str) -> &'static str {
    match operation {
        "exec" => "DB.EXEC_CONFLICT",
        "execTx" => "DB.EXEC_TX_CONFLICT",
        "queryOne" => "DB.QUERY_ONE_CONFLICT",
        _ => "DB.OPERATION_CONFLICT",
    }
}

fn lasm_db_operation_unavailable_code(operation: &str) -> &'static str {
    match operation {
        "exec" => "DB.EXEC_UNAVAILABLE",
        "execTx" => "DB.EXEC_TX_UNAVAILABLE",
        "queryOne" => "DB.QUERY_ONE_UNAVAILABLE",
        _ => "DB.OPERATION_UNAVAILABLE",
    }
}

fn extract_lasm_db_runtime_sqlstate(normalized_message: &str) -> Option<&str> {
    let marker = "sqlstate=";
    let start = normalized_message.find(marker)? + marker.len();
    let tail = &normalized_message[start..];
    let end = tail
        .find(|ch: char| !ch.is_ascii_alphanumeric())
        .unwrap_or(tail.len());
    let code = &tail[..end];
    if code.len() == 5 && code.chars().all(|ch| ch.is_ascii_alphanumeric()) {
        Some(code)
    } else {
        None
    }
}

fn extract_lasm_db_runtime_sqlite_code(normalized_message: &str) -> Option<&str> {
    let marker = "sqlite_code=";
    let start = normalized_message.find(marker)? + marker.len();
    let tail = &normalized_message[start..];
    let end = tail
        .find(|ch: char| !ch.is_ascii_alphanumeric())
        .unwrap_or(tail.len());
    let code = &tail[..end];
    if code.is_empty() {
        None
    } else {
        Some(code)
    }
}

fn extract_lasm_db_runtime_sqlite_extended_code(normalized_message: &str) -> Option<i32> {
    let marker = "sqlite_extended_code=";
    let start = normalized_message.find(marker)? + marker.len();
    let tail = &normalized_message[start..];
    let end = tail
        .find(|ch: char| !ch.is_ascii_digit())
        .unwrap_or(tail.len());
    if end == 0 {
        return None;
    }
    tail[..end].parse::<i32>().ok()
}

fn is_lasm_db_runtime_postgres_stale_plan_error(normalized_message: &str) -> bool {
    let has_prepared_statement_not_found = normalized_message.contains("prepared statement")
        && normalized_message.contains("does not exist");
    let has_cached_plan_shape_drift =
        normalized_message.contains("cached plan must not change result type");
    has_prepared_statement_not_found || has_cached_plan_shape_drift
}

pub(crate) fn classify_lasm_db_runtime_error(
    operation: &str,
    message: &str,
) -> (u16, &'static str, &'static str) {
    let normalized = message.to_ascii_lowercase();
    if normalized.contains("requires sec4_db_alpha_db_postgres_dsn")
        || normalized.contains("requires sec4_rt_lasm_db_postgres_dsn")
        || normalized.contains("sqlite records store path unavailable")
        || normalized.contains("sqlite records store connection unavailable")
        || normalized.contains("could not connect lasm dynamic postgres records store")
        || normalized.contains("native tls connector bootstrap failed")
        || normalized.contains("could not create lasm dynamic sqlite records store directory")
        || normalized.contains("could not open lasm dynamic sqlite records store")
    {
        return (500, "DB.ADAPTER_CONFIG_INVALID", "internal");
    }
    if is_lasm_db_runtime_postgres_stale_plan_error(normalized.as_str()) {
        return (409, lasm_db_operation_conflict_code(operation), "conflict");
    }
    if let Some(sqlstate) = extract_lasm_db_runtime_sqlstate(normalized.as_str()) {
        match sqlstate {
            "57014" => return (504, lasm_db_operation_timeout_code(operation), "timeout"),
            "55p03" | "40001" | "40p01" | "25p02" => {
                return (
                    409,
                    lasm_db_operation_lock_conflict_code(operation),
                    "conflict",
                );
            }
            _ if sqlstate.starts_with("55") => {
                return (
                    409,
                    lasm_db_operation_lock_conflict_code(operation),
                    "conflict",
                );
            }
            "26000" => return (409, lasm_db_operation_conflict_code(operation), "conflict"),
            "23505" => return (409, lasm_db_operation_conflict_code(operation), "conflict"),
            "23502" | "23514" | "42601" | "42703" => {
                return (
                    400,
                    lasm_db_operation_validation_code(operation),
                    "validation",
                );
            }
            "0a000" => {
                return (
                    400,
                    lasm_db_operation_validation_code(operation),
                    "validation",
                );
            }
            _ if sqlstate.starts_with("22") => {
                return (
                    400,
                    lasm_db_operation_validation_code(operation),
                    "validation",
                );
            }
            _ if sqlstate.starts_with("23") => {
                return (
                    400,
                    lasm_db_operation_validation_code(operation),
                    "validation",
                );
            }
            _ if sqlstate.starts_with("42") => {
                return (
                    400,
                    lasm_db_operation_validation_code(operation),
                    "validation",
                );
            }
            _ if sqlstate.starts_with("3f") => {
                return (
                    400,
                    lasm_db_operation_validation_code(operation),
                    "validation",
                );
            }
            _ if sqlstate.starts_with("28") || sqlstate == "3d000" => {
                return (500, "DB.ADAPTER_CONFIG_INVALID", "internal");
            }
            _ if sqlstate.starts_with("08") => {
                return (
                    503,
                    lasm_db_operation_unavailable_code(operation),
                    "missing_dependency",
                );
            }
            _ if sqlstate.starts_with("53")
                || sqlstate.starts_with("54")
                || sqlstate.starts_with("57")
                || sqlstate.starts_with("58") =>
            {
                return (
                    503,
                    lasm_db_operation_unavailable_code(operation),
                    "missing_dependency",
                );
            }
            _ => {}
        }
    }
    if let Some(sqlite_code) = extract_lasm_db_runtime_sqlite_code(normalized.as_str()) {
        let sqlite_extended_code =
            extract_lasm_db_runtime_sqlite_extended_code(normalized.as_str());
        match sqlite_code {
            "databasebusy" | "databaselocked" => {
                return (
                    409,
                    lasm_db_operation_lock_conflict_code(operation),
                    "conflict",
                );
            }
            "constraintviolation" => {
                if matches!(sqlite_extended_code, Some(1555 | 2067)) {
                    return (409, lasm_db_operation_conflict_code(operation), "conflict");
                }
                return (
                    400,
                    lasm_db_operation_validation_code(operation),
                    "validation",
                );
            }
            "toobig" | "typemismatch" | "parameteroutofrange" => {
                return (
                    400,
                    lasm_db_operation_validation_code(operation),
                    "validation",
                );
            }
            "permissiondenied" | "readonly" | "cannotopen" | "notadatabase" => {
                return (500, "DB.ADAPTER_CONFIG_INVALID", "internal");
            }
            "systemiofailure" | "diskfull" | "outofmemory" | "schemachanged" => {
                return (
                    503,
                    lasm_db_operation_unavailable_code(operation),
                    "missing_dependency",
                );
            }
            _ => {}
        }
    }
    if let Some(sqlite_extended_code) =
        extract_lasm_db_runtime_sqlite_extended_code(normalized.as_str())
    {
        match sqlite_extended_code {
            5 | 6 => {
                return (
                    409,
                    lasm_db_operation_lock_conflict_code(operation),
                    "conflict",
                )
            }
            1555 | 2067 => return (409, lasm_db_operation_conflict_code(operation), "conflict"),
            1299 | 275 | 3091 | 3094 => {
                return (
                    400,
                    lasm_db_operation_validation_code(operation),
                    "validation",
                );
            }
            8 | 14 | 26 => return (500, "DB.ADAPTER_CONFIG_INVALID", "internal"),
            7 | 10 | 13 => {
                return (
                    503,
                    lasm_db_operation_unavailable_code(operation),
                    "missing_dependency",
                );
            }
            _ => {}
        }
    }
    if normalized.contains("canceling statement due to statement timeout")
        || normalized.contains("connect timeout")
        || normalized.contains("connection timed out")
        || normalized.contains("timeout expired")
    {
        return (504, lasm_db_operation_timeout_code(operation), "timeout");
    }
    if normalized.contains("canceling statement due to lock timeout")
        || normalized.contains("database is locked")
        || normalized.contains("could not serialize access due to")
        || normalized.contains("deadlock detected")
        || normalized.contains("current transaction is aborted")
    {
        return (
            409,
            lasm_db_operation_lock_conflict_code(operation),
            "conflict",
        );
    }
    if normalized.contains("requires at least")
        || normalized.contains("expects exactly")
        || normalized.contains("requires select-style sql statement")
        || normalized.contains("requires row-returning sql statement")
        || normalized.contains("requires non-empty sql statement")
        || normalized.contains("requires a single sql statement")
        || normalized.contains("requires named parameter")
        || normalized.contains("requires sql placeholders to be named")
        || normalized.contains("is not present in sql statement")
    {
        return (
            400,
            lasm_db_operation_validation_code(operation),
            "validation",
        );
    }
    if normalized.contains("not null constraint failed")
        || normalized.contains("check constraint failed")
        || normalized.contains("violates not-null constraint")
        || normalized.contains("violates check constraint")
        || normalized.contains("invalid input syntax for")
        || normalized.contains("syntax error at or near")
        || normalized.contains("syntax error")
        || normalized.contains("unrecognized token")
        || normalized.contains("no such column")
        || normalized.contains("column does not exist")
        || normalized.contains("feature not supported")
    {
        return (
            400,
            lasm_db_operation_validation_code(operation),
            "validation",
        );
    }
    if normalized.contains("connection refused")
        || normalized.contains("could not connect to server")
        || normalized.contains("server closed the connection unexpectedly")
        || normalized.contains("connection reset by peer")
        || normalized.contains("too many connections")
        || normalized.contains("remaining connection slots are reserved")
        || normalized.contains("broken pipe")
    {
        return (
            503,
            lasm_db_operation_unavailable_code(operation),
            "missing_dependency",
        );
    }
    if normalized.contains("unique constraint failed")
        || normalized.contains("duplicate key value violates unique constraint")
    {
        return (409, lasm_db_operation_conflict_code(operation), "conflict");
    }
    let code = match operation {
        "exec" => "DB.EXEC_FAILED",
        "execTx" => "DB.EXEC_TX_FAILED",
        "queryOne" => "DB.QUERY_ONE_FAILED",
        _ => "DB.OPERATION_FAILED",
    };
    (500, code, "internal")
}

pub(crate) fn parse_lasm_positive_i64(value: &str) -> Option<i64> {
    value
        .trim()
        .parse::<i64>()
        .ok()
        .filter(|candidate| *candidate > 0)
}

pub(crate) fn is_lasm_valid_db_cap_handle(db: i64) -> bool {
    db == 1
}

pub(crate) fn normalize_lasm_db_params(value: &str) -> String {
    let (normalized, _) = normalize_lasm_db_params_and_value(value);
    normalized
}

pub(crate) fn normalize_lasm_db_params_and_value(
    value: &str,
) -> (String, Option<serde_json::Value>) {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed == "0" {
        return ("0".to_string(), None);
    }
    let parsed = match serde_json::from_str::<serde_json::Value>(trimmed) {
        Ok(parsed) => parsed,
        Err(_) => return (trimmed.to_string(), None),
    };
    let canonical = canonicalize_lasm_db_params_value(parsed);
    let normalized = serde_json::to_string(&canonical).unwrap_or_else(|_| trimmed.to_string());
    (normalized, Some(canonical))
}

fn canonicalize_lasm_db_params_value(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Array(values) => serde_json::Value::Array(
            values
                .into_iter()
                .map(canonicalize_lasm_db_params_value)
                .collect(),
        ),
        serde_json::Value::Object(entries) => {
            let mut ordered = BTreeMap::new();
            for (key, entry) in entries {
                ordered.insert(key, canonicalize_lasm_db_params_value(entry));
            }
            let mut canonical = serde_json::Map::with_capacity(ordered.len());
            for (key, entry) in ordered {
                canonical.insert(key, entry);
            }
            serde_json::Value::Object(canonical)
        }
        other => other,
    }
}

fn rebuild_lasm_db_records_tracking(state: &mut LasmDynamicResponseState) {
    state.db_record_signatures.clear();
    state.db_latest_record_by_signature.clear();
    for record in state.db_records.iter() {
        let signature = crate::lasm_dynamic_state::lasm_db_record_signature_key(
            record.db,
            &record.template,
            &record.params,
        );
        *state
            .db_record_signatures
            .entry(signature.clone())
            .or_insert(0) += 1;
        state
            .db_latest_record_by_signature
            .insert(signature, record.clone());
    }
    let next_db_record_id = state
        .db_records
        .iter()
        .map(|record| record.id)
        .max()
        .unwrap_or(0)
        .saturating_add(1);
    state.next_db_record_id = next_db_record_id;
}

fn truncate_lasm_db_records_for_capacity(
    state: &mut LasmDynamicResponseState,
) -> Vec<LasmDbRecord> {
    let bounded_capacity = state.db_records_max.max(1);
    if state.db_records.len() <= bounded_capacity {
        return Vec::new();
    }
    let overflow = state.db_records.len() - bounded_capacity;
    state.db_records.drain(0..overflow).collect()
}

fn bootstrap_lasm_dynamic_db_records_from_postgres(
    state: &mut LasmDynamicResponseState,
) -> Result<(), String> {
    if state.db_records_postgres_bootstrapped {
        return Ok(());
    }
    if state.db_records.is_empty() {
        let records = {
            let client = state
                .db_records_postgres_client
                .as_mut()
                .ok_or_else(|| LASM_DB_POSTGRES_DSN_CONFIG_ERROR_MESSAGE.to_string())?;
            load_lasm_dynamic_db_records_from_postgres(client)?
        };
        state.db_records = records;
    }
    let dropped_records = truncate_lasm_db_records_for_capacity(state);
    if !dropped_records.is_empty() {
        state.db_records_dropped_total = state
            .db_records_dropped_total
            .saturating_add(dropped_records.len() as u64);
    }
    rebuild_lasm_db_records_tracking(state);
    state.db_records_postgres_bootstrapped = true;
    Ok(())
}

pub(crate) fn allocate_lasm_db_tx_handle(
    state: &mut LasmDynamicResponseState,
    db: i64,
) -> Option<i64> {
    if state.db_tx_handles.len() >= state.db_tx_max_handles {
        return None;
    }
    let mut tx = state.next_db_tx_handle.max(1);
    let start_tx = tx;
    loop {
        if let std::collections::hash_map::Entry::Vacant(entry) = state.db_tx_handles.entry(tx) {
            entry.insert(LasmDbTxState { db, active: false });
            state.next_db_tx_handle = if tx == i64::MAX { 1 } else { tx + 1 };
            return Some(tx);
        }
        tx = if tx == i64::MAX { 1 } else { tx + 1 };
        if tx == start_tx {
            return None;
        }
    }
}

pub(crate) fn lasm_dynamic_postgres_client_mut(
    state: &mut LasmDynamicResponseState,
) -> Result<&mut PostgresClient, String> {
    if state.db_records_postgres_client.is_none() {
        reconnect_lasm_dynamic_postgres_client(state)?;
    }
    if !state.db_records_postgres_bootstrapped {
        bootstrap_lasm_dynamic_db_records_from_postgres(state)?;
    }
    state
        .db_records_postgres_client
        .as_mut()
        .ok_or_else(|| LASM_DB_POSTGRES_DSN_CONFIG_ERROR_MESSAGE.to_string())
}

#[allow(dead_code)]
pub(crate) fn lasm_dynamic_postgres_prepared_statement(
    state: &mut LasmDynamicResponseState,
    query_template: &str,
) -> Result<PostgresStatement, String> {
    if let Some(statement) = state
        .db_records_postgres_statement_cache
        .get(query_template)
        .cloned()
    {
        touch_lasm_bounded_cache_entry(
            &mut state.db_records_postgres_statement_cache_order,
            query_template,
        );
        return Ok(statement);
    }
    let statement = {
        let client = lasm_dynamic_postgres_client_mut(state)?;
        client.prepare(query_template).map_err(|err| {
            if let Some(sqlstate) = err.code().map(|code| code.code()) {
                return format!("postgres prepare failed: {err}; sqlstate={sqlstate}");
            }
            format!("postgres prepare failed: {err}")
        })?
    };
    let evicted = insert_lasm_bounded_cache_entry(
        &mut state.db_records_postgres_statement_cache,
        &mut state.db_records_postgres_statement_cache_order,
        state.db_postgres_statement_cache_max,
        query_template.to_string(),
        statement.clone(),
    );
    state.db_postgres_statement_cache_evictions_total = state
        .db_postgres_statement_cache_evictions_total
        .saturating_add(evicted);
    Ok(statement)
}

#[allow(dead_code)]
pub(crate) fn insert_lasm_bounded_cache_entry<V>(
    cache: &mut HashMap<String, V>,
    order: &mut VecDeque<String>,
    capacity: usize,
    key: String,
    value: V,
) -> u64 {
    let bounded_capacity = capacity.max(1);
    let key_exists = cache.contains_key(key.as_str());
    let mut evicted = 0_u64;
    if !key_exists {
        while cache.len() >= bounded_capacity {
            let Some(evicted_key) = order.pop_front() else {
                cache.clear();
                break;
            };
            if cache.remove(evicted_key.as_str()).is_some() {
                evicted = evicted.saturating_add(1);
            }
        }
    }
    cache.insert(key.clone(), value);
    if !key_exists {
        order.push_back(key);
    }
    evicted
}

#[allow(dead_code)]
pub(crate) fn touch_lasm_bounded_cache_entry(order: &mut VecDeque<String>, key: &str) {
    if order.back().map(|value| value.as_str()) == Some(key) {
        return;
    }
    if let Some(index) = order.iter().position(|value| value == key) {
        order.remove(index);
    }
    order.push_back(key.to_string());
}

pub(crate) fn reconnect_lasm_dynamic_postgres_client(
    state: &mut LasmDynamicResponseState,
) -> Result<(), String> {
    let dsn = state
        .db_records_postgres_dsn
        .as_deref()
        .ok_or_else(|| LASM_DB_POSTGRES_DSN_CONFIG_ERROR_MESSAGE.to_string())?;
    let mut client = connect_lasm_dynamic_db_records_postgres(
        dsn,
        state.db_postgres_tls_mode,
        state.db_postgres_statement_timeout_ms.max(1),
        state.db_postgres_lock_timeout_ms.max(1),
        state.db_postgres_connect_timeout_ms.max(1),
    )?;
    ensure_lasm_dynamic_db_records_postgres_schema(&mut client)?;
    state.db_records_postgres_client = Some(client);
    state.db_records_postgres_bootstrapped = false;
    state.db_records_postgres_statement_cache.clear();
    state.db_records_postgres_statement_cache_order.clear();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        classify_lasm_db_runtime_error, insert_lasm_bounded_cache_entry, normalize_lasm_db_params,
        touch_lasm_bounded_cache_entry,
    };
    use std::collections::{HashMap, VecDeque};

    #[test]
    fn classify_db_runtime_statement_timeout_error() {
        let (status, code, kind) =
            classify_lasm_db_runtime_error("exec", "canceling statement due to statement timeout");
        assert_eq!(status, 504);
        assert_eq!(code, "DB.EXEC_TIMEOUT");
        assert_eq!(kind, "timeout");
    }

    #[test]
    fn classify_db_runtime_lock_timeout_error() {
        let (status, code, kind) = classify_lasm_db_runtime_error("queryOne", "database is locked");
        assert_eq!(status, 409);
        assert_eq!(code, "DB.QUERY_ONE_LOCK_TIMEOUT");
        assert_eq!(kind, "conflict");
    }

    #[test]
    fn classify_db_runtime_postgres_retryable_conflict_as_conflict() {
        let (status, code, kind) = classify_lasm_db_runtime_error(
            "execTx",
            "ERROR: could not serialize access due to concurrent update",
        );
        assert_eq!(status, 409);
        assert_eq!(code, "DB.EXEC_TX_LOCK_TIMEOUT");
        assert_eq!(kind, "conflict");
    }

    #[test]
    fn classify_db_runtime_row_returning_shape_error_as_validation() {
        let (status, code, kind) = classify_lasm_db_runtime_error(
            "queryOne",
            "sqlite queryOne requires row-returning SQL statement (SELECT/WITH/VALUES/TABLE or DML ... RETURNING)",
        );
        assert_eq!(status, 400);
        assert_eq!(code, "DB.QUERY_ONE_INVALID");
        assert_eq!(kind, "validation");
    }

    #[test]
    fn classify_db_runtime_named_param_errors_as_validation() {
        let (status, code, kind) = classify_lasm_db_runtime_error(
            "exec",
            "sqlite query requires named parameter `role` in params object",
        );
        assert_eq!(status, 400);
        assert_eq!(code, "DB.EXEC_INVALID");
        assert_eq!(kind, "validation");
    }

    #[test]
    fn classify_db_runtime_exact_arity_errors_as_validation() {
        let (status, code, kind) = classify_lasm_db_runtime_error(
            "exec",
            "sqlite query expects exactly 1 sql parameters but received 2",
        );
        assert_eq!(status, 400);
        assert_eq!(code, "DB.EXEC_INVALID");
        assert_eq!(kind, "validation");
    }

    #[test]
    fn classify_db_runtime_sql_syntax_errors_as_validation() {
        let (status, code, kind) =
            classify_lasm_db_runtime_error("exec", "syntax error at or near \"FROM\"");
        assert_eq!(status, 400);
        assert_eq!(code, "DB.EXEC_INVALID");
        assert_eq!(kind, "validation");
    }

    #[test]
    fn classify_db_runtime_missing_column_errors_as_validation() {
        let (status, code, kind) =
            classify_lasm_db_runtime_error("queryOne", "no such column: unknown_field");
        assert_eq!(status, 400);
        assert_eq!(code, "DB.QUERY_ONE_INVALID");
        assert_eq!(kind, "validation");
    }

    #[test]
    fn classify_db_runtime_sqlite_connection_unavailable_as_adapter_config_invalid() {
        let (status, code, kind) =
            classify_lasm_db_runtime_error("exec", "sqlite records store connection unavailable");
        assert_eq!(status, 500);
        assert_eq!(code, "DB.ADAPTER_CONFIG_INVALID");
        assert_eq!(kind, "internal");
    }

    #[test]
    fn classify_db_runtime_postgres_connect_errors_as_adapter_config_invalid() {
        let (status, code, kind) = classify_lasm_db_runtime_error(
            "queryOne",
            "could not connect LASM dynamic postgres records store: Connection refused",
        );
        assert_eq!(status, 500);
        assert_eq!(code, "DB.ADAPTER_CONFIG_INVALID");
        assert_eq!(kind, "internal");
    }

    #[test]
    fn classify_db_runtime_connect_timeout_error() {
        let (status, code, kind) =
            classify_lasm_db_runtime_error("exec", "postgres execution failed: connect timeout");
        assert_eq!(status, 504);
        assert_eq!(code, "DB.EXEC_TIMEOUT");
        assert_eq!(kind, "timeout");
    }

    #[test]
    fn classify_db_runtime_connect_refused_error_as_unavailable() {
        let (status, code, kind) = classify_lasm_db_runtime_error(
            "execTx",
            "postgres transaction execution failed: could not connect to server: Connection refused",
        );
        assert_eq!(status, 503);
        assert_eq!(code, "DB.EXEC_TX_UNAVAILABLE");
        assert_eq!(kind, "missing_dependency");
    }

    #[test]
    fn classify_db_runtime_too_many_connections_as_unavailable() {
        let (status, code, kind) = classify_lasm_db_runtime_error(
            "queryOne",
            "FATAL: remaining connection slots are reserved for non-replication superuser connections",
        );
        assert_eq!(status, 503);
        assert_eq!(code, "DB.QUERY_ONE_UNAVAILABLE");
        assert_eq!(kind, "missing_dependency");
    }

    #[test]
    fn classify_db_runtime_case_insensitive_constraint_errors_as_validation() {
        let (status, code, kind) =
            classify_lasm_db_runtime_error("queryOne", "NOT NULL CONSTRAINT FAILED: users.email");
        assert_eq!(status, 400);
        assert_eq!(code, "DB.QUERY_ONE_INVALID");
        assert_eq!(kind, "validation");
    }

    #[test]
    fn classify_db_runtime_sqlstate_timeout_error() {
        let (status, code, kind) = classify_lasm_db_runtime_error(
            "exec",
            "postgres execution failed: query canceled; sqlstate=57014",
        );
        assert_eq!(status, 504);
        assert_eq!(code, "DB.EXEC_TIMEOUT");
        assert_eq!(kind, "timeout");
    }

    #[test]
    fn classify_db_runtime_sqlstate_conflict_error() {
        let (status, code, kind) = classify_lasm_db_runtime_error(
            "execTx",
            "postgres transaction execution failed: deadlock detected; sqlstate=40P01",
        );
        assert_eq!(status, 409);
        assert_eq!(code, "DB.EXEC_TX_LOCK_TIMEOUT");
        assert_eq!(kind, "conflict");
    }

    #[test]
    fn classify_db_runtime_sqlstate_object_state_class_as_conflict() {
        let (status, code, kind) = classify_lasm_db_runtime_error(
            "queryOne",
            "postgres queryOne execution failed: object in use; sqlstate=55006",
        );
        assert_eq!(status, 409);
        assert_eq!(code, "DB.QUERY_ONE_LOCK_TIMEOUT");
        assert_eq!(kind, "conflict");
    }

    #[test]
    fn classify_db_runtime_sqlstate_in_failed_transaction_as_conflict() {
        let (status, code, kind) = classify_lasm_db_runtime_error(
            "execTx",
            "postgres transaction execution failed: current transaction is aborted; sqlstate=25P02",
        );
        assert_eq!(status, 409);
        assert_eq!(code, "DB.EXEC_TX_LOCK_TIMEOUT");
        assert_eq!(kind, "conflict");
    }

    #[test]
    fn classify_db_runtime_transaction_aborted_message_as_conflict() {
        let (status, code, kind) = classify_lasm_db_runtime_error(
            "queryOne",
            "postgres queryOne execution failed: current transaction is aborted, commands ignored until end of transaction block",
        );
        assert_eq!(status, 409);
        assert_eq!(code, "DB.QUERY_ONE_LOCK_TIMEOUT");
        assert_eq!(kind, "conflict");
    }

    #[test]
    fn classify_db_runtime_sqlstate_validation_error() {
        let (status, code, kind) = classify_lasm_db_runtime_error(
            "queryOne",
            "postgres queryOne execution failed: invalid input syntax for type integer; sqlstate=22P02",
        );
        assert_eq!(status, 400);
        assert_eq!(code, "DB.QUERY_ONE_INVALID");
        assert_eq!(kind, "validation");
    }

    #[test]
    fn classify_db_runtime_stale_prepared_statement_error_as_conflict() {
        let (status, code, kind) = classify_lasm_db_runtime_error(
            "exec",
            "postgres execution failed after stale prepared statement refresh: prepared statement \"s1\" does not exist; sqlstate=26000",
        );
        assert_eq!(status, 409);
        assert_eq!(code, "DB.EXEC_CONFLICT");
        assert_eq!(kind, "conflict");
    }

    #[test]
    fn classify_db_runtime_sqlstate_invalid_statement_name_as_conflict() {
        let (status, code, kind) = classify_lasm_db_runtime_error(
            "execTx",
            "postgres transaction execution failed after prepared refresh; sqlstate=26000",
        );
        assert_eq!(status, 409);
        assert_eq!(code, "DB.EXEC_TX_CONFLICT");
        assert_eq!(kind, "conflict");
    }

    #[test]
    fn classify_db_runtime_cached_plan_shape_drift_as_conflict() {
        let (status, code, kind) = classify_lasm_db_runtime_error(
            "queryOne",
            "postgres queryOne execution failed after stale prepared statement refresh: cached plan must not change result type; sqlstate=0A000",
        );
        assert_eq!(status, 409);
        assert_eq!(code, "DB.QUERY_ONE_CONFLICT");
        assert_eq!(kind, "conflict");
    }

    #[test]
    fn classify_db_runtime_sqlstate_feature_not_supported_as_validation() {
        let (status, code, kind) = classify_lasm_db_runtime_error(
            "exec",
            "postgres execution failed: feature not supported; sqlstate=0A000",
        );
        assert_eq!(status, 400);
        assert_eq!(code, "DB.EXEC_INVALID");
        assert_eq!(kind, "validation");
    }

    #[test]
    fn classify_db_runtime_feature_not_supported_message_as_validation() {
        let (status, code, kind) = classify_lasm_db_runtime_error(
            "queryOne",
            "postgres queryOne execution failed: feature not supported",
        );
        assert_eq!(status, 400);
        assert_eq!(code, "DB.QUERY_ONE_INVALID");
        assert_eq!(kind, "validation");
    }

    #[test]
    fn classify_db_runtime_sqlstate_unavailable_error() {
        let (status, code, kind) = classify_lasm_db_runtime_error(
            "queryOne",
            "postgres queryOne execution failed: too many connections for role; sqlstate=53300",
        );
        assert_eq!(status, 503);
        assert_eq!(code, "DB.QUERY_ONE_UNAVAILABLE");
        assert_eq!(kind, "missing_dependency");
    }

    #[test]
    fn classify_db_runtime_sqlstate_connection_exception_as_unavailable() {
        let (status, code, kind) = classify_lasm_db_runtime_error(
            "exec",
            "postgres execution failed: connection failure; sqlstate=08006",
        );
        assert_eq!(status, 503);
        assert_eq!(code, "DB.EXEC_UNAVAILABLE");
        assert_eq!(kind, "missing_dependency");
    }

    #[test]
    fn classify_db_runtime_sqlstate_resource_class_as_unavailable() {
        let (status, code, kind) = classify_lasm_db_runtime_error(
            "queryOne",
            "postgres queryOne execution failed: disk full; sqlstate=53100",
        );
        assert_eq!(status, 503);
        assert_eq!(code, "DB.QUERY_ONE_UNAVAILABLE");
        assert_eq!(kind, "missing_dependency");
    }

    #[test]
    fn classify_db_runtime_sqlstate_program_limit_class_as_unavailable() {
        let (status, code, kind) = classify_lasm_db_runtime_error(
            "exec",
            "postgres execution failed: program limit exceeded; sqlstate=54000",
        );
        assert_eq!(status, 503);
        assert_eq!(code, "DB.EXEC_UNAVAILABLE");
        assert_eq!(kind, "missing_dependency");
    }

    #[test]
    fn classify_db_runtime_sqlstate_system_error_class_as_unavailable() {
        let (status, code, kind) = classify_lasm_db_runtime_error(
            "execTx",
            "postgres transaction execution failed: io error; sqlstate=58030",
        );
        assert_eq!(status, 503);
        assert_eq!(code, "DB.EXEC_TX_UNAVAILABLE");
        assert_eq!(kind, "missing_dependency");
    }

    #[test]
    fn classify_db_runtime_sqlstate_auth_error_as_adapter_config_invalid() {
        let (status, code, kind) = classify_lasm_db_runtime_error(
            "execTx",
            "postgres transaction execution failed: password authentication failed; sqlstate=28P01",
        );
        assert_eq!(status, 500);
        assert_eq!(code, "DB.ADAPTER_CONFIG_INVALID");
        assert_eq!(kind, "internal");
    }

    #[test]
    fn classify_db_runtime_sqlstate_access_rule_error_as_validation() {
        let (status, code, kind) = classify_lasm_db_runtime_error(
            "queryOne",
            "postgres queryOne execution failed: relation does not exist; sqlstate=42P01",
        );
        assert_eq!(status, 400);
        assert_eq!(code, "DB.QUERY_ONE_INVALID");
        assert_eq!(kind, "validation");
    }

    #[test]
    fn classify_db_runtime_sqlstate_schema_name_error_as_validation() {
        let (status, code, kind) = classify_lasm_db_runtime_error(
            "execTx",
            "postgres transaction execution failed: schema does not exist; sqlstate=3F000",
        );
        assert_eq!(status, 400);
        assert_eq!(code, "DB.EXEC_TX_INVALID");
        assert_eq!(kind, "validation");
    }

    #[test]
    fn classify_db_runtime_sqlstate_prepare_validation_error() {
        let (status, code, kind) = classify_lasm_db_runtime_error(
            "exec",
            "postgres prepare failed: syntax error at or near \"FROM\"; sqlstate=42601",
        );
        assert_eq!(status, 400);
        assert_eq!(code, "DB.EXEC_INVALID");
        assert_eq!(kind, "validation");
    }

    #[test]
    fn classify_db_runtime_sqlite_code_lock_conflict_error() {
        let (status, code, kind) = classify_lasm_db_runtime_error(
            "queryOne",
            "sqlite queryOne execution failed: database is locked; sqlite_code=DatabaseLocked; sqlite_extended_code=5",
        );
        assert_eq!(status, 409);
        assert_eq!(code, "DB.QUERY_ONE_LOCK_TIMEOUT");
        assert_eq!(kind, "conflict");
    }

    #[test]
    fn classify_db_runtime_sqlite_code_adapter_config_error() {
        let (status, code, kind) = classify_lasm_db_runtime_error(
            "exec",
            "sqlite execution prepare failed: attempt to write a readonly database; sqlite_code=ReadOnly; sqlite_extended_code=8",
        );
        assert_eq!(status, 500);
        assert_eq!(code, "DB.ADAPTER_CONFIG_INVALID");
        assert_eq!(kind, "internal");
    }

    #[test]
    fn classify_db_runtime_sqlite_unique_constraint_error_as_conflict() {
        let (status, code, kind) = classify_lasm_db_runtime_error(
            "exec",
            "sqlite execution failed: UNIQUE constraint failed: users.email; sqlite_code=ConstraintViolation; sqlite_extended_code=2067",
        );
        assert_eq!(status, 409);
        assert_eq!(code, "DB.EXEC_CONFLICT");
        assert_eq!(kind, "conflict");
    }

    #[test]
    fn classify_db_runtime_sqlite_not_null_constraint_error_as_validation() {
        let (status, code, kind) = classify_lasm_db_runtime_error(
            "queryOne",
            "sqlite queryOne execution failed: NOT NULL constraint failed: users.email; sqlite_code=ConstraintViolation; sqlite_extended_code=1299",
        );
        assert_eq!(status, 400);
        assert_eq!(code, "DB.QUERY_ONE_INVALID");
        assert_eq!(kind, "validation");
    }

    #[test]
    fn classify_db_runtime_sqlite_extended_code_conflict_without_sqlite_code_marker() {
        let (status, code, kind) = classify_lasm_db_runtime_error(
            "execTx",
            "sqlite execution failed: UNIQUE constraint failed: users.email; sqlite_extended_code=2067",
        );
        assert_eq!(status, 409);
        assert_eq!(code, "DB.EXEC_TX_CONFLICT");
        assert_eq!(kind, "conflict");
    }

    #[test]
    fn classify_db_runtime_unknown_failure_defaults_to_internal_kind() {
        let (status, code, kind) =
            classify_lasm_db_runtime_error("exec", "unexpected adapter failure details");
        assert_eq!(status, 500);
        assert_eq!(code, "DB.EXEC_FAILED");
        assert_eq!(kind, "internal");
    }

    #[test]
    fn bounded_cache_entry_evicts_oldest_single_entry_when_full() {
        let mut cache = HashMap::new();
        let mut order = VecDeque::new();

        let first_evicted =
            insert_lasm_bounded_cache_entry(&mut cache, &mut order, 2, "a".to_string(), 1_i32);
        let second_evicted =
            insert_lasm_bounded_cache_entry(&mut cache, &mut order, 2, "b".to_string(), 2_i32);
        let third_evicted =
            insert_lasm_bounded_cache_entry(&mut cache, &mut order, 2, "c".to_string(), 3_i32);

        assert_eq!(first_evicted, 0);
        assert_eq!(second_evicted, 0);
        assert_eq!(third_evicted, 1);
        assert_eq!(cache.len(), 2);
        assert!(!cache.contains_key("a"));
        assert_eq!(cache.get("b"), Some(&2));
        assert_eq!(cache.get("c"), Some(&3));
    }

    #[test]
    fn touch_cache_entry_refreshes_recency_for_next_eviction() {
        let mut cache = HashMap::new();
        let mut order = VecDeque::new();

        insert_lasm_bounded_cache_entry(&mut cache, &mut order, 2, "a".to_string(), 1_i32);
        insert_lasm_bounded_cache_entry(&mut cache, &mut order, 2, "b".to_string(), 2_i32);
        touch_lasm_bounded_cache_entry(&mut order, "a");
        insert_lasm_bounded_cache_entry(&mut cache, &mut order, 2, "c".to_string(), 3_i32);

        assert_eq!(cache.len(), 2);
        assert_eq!(cache.get("a"), Some(&1));
        assert_eq!(cache.get("c"), Some(&3));
        assert!(!cache.contains_key("b"));
    }

    #[test]
    fn normalize_db_params_canonicalizes_object_key_order() {
        let normalized = normalize_lasm_db_params("{\"b\":2,\"a\":1}");
        assert_eq!(normalized, "{\"a\":1,\"b\":2}");
    }

    #[test]
    fn normalize_db_params_canonicalizes_nested_json() {
        let normalized =
            normalize_lasm_db_params("{\"b\":[{\"z\":1,\"a\":2}],\"a\":{\"y\":3,\"x\":4}}");
        assert_eq!(
            normalized,
            "{\"a\":{\"x\":4,\"y\":3},\"b\":[{\"a\":2,\"z\":1}]}"
        );
    }

    #[test]
    fn normalize_db_params_preserves_non_json_text() {
        let normalized = normalize_lasm_db_params("alpha");
        assert_eq!(normalized, "alpha");
    }
}
