use crate::lasm_db_adapter_state::{
    connect_lasm_dynamic_db_records_postgres, ensure_lasm_dynamic_db_records_postgres_schema,
};
use crate::LasmDynamicResponseState;
use postgres::{Client as PostgresClient, Statement as PostgresStatement};
use std::collections::{BTreeMap, HashMap, VecDeque};

pub(crate) fn classify_lasm_db_runtime_error(
    operation: &str,
    message: &str,
) -> (u16, &'static str, &'static str) {
    if message.contains("requires SEC4_RT_LASM_DB_POSTGRES_DSN")
        || message.contains("sqlite records store path unavailable")
    {
        return (500, "DB.ADAPTER_CONFIG_INVALID", "internal");
    }
    if message.contains("canceling statement due to statement timeout") {
        let code = match operation {
            "exec" => "DB.EXEC_TIMEOUT",
            "execTx" => "DB.EXEC_TX_TIMEOUT",
            "queryOne" => "DB.QUERY_ONE_TIMEOUT",
            _ => "DB.OPERATION_TIMEOUT",
        };
        return (504, code, "timeout");
    }
    if message.contains("canceling statement due to lock timeout")
        || message.contains("database is locked")
    {
        let code = match operation {
            "exec" => "DB.EXEC_LOCK_TIMEOUT",
            "execTx" => "DB.EXEC_TX_LOCK_TIMEOUT",
            "queryOne" => "DB.QUERY_ONE_LOCK_TIMEOUT",
            _ => "DB.OPERATION_LOCK_TIMEOUT",
        };
        return (409, code, "conflict");
    }
    if message.contains("requires at least")
        || message.contains("requires SELECT-style SQL statement")
        || message.contains("requires non-empty SQL statement")
        || message.contains("requires a single SQL statement")
    {
        let code = match operation {
            "exec" => "DB.EXEC_INVALID",
            "execTx" => "DB.EXEC_TX_INVALID",
            "queryOne" => "DB.QUERY_ONE_INVALID",
            _ => "DB.OPERATION_INVALID",
        };
        return (400, code, "validation");
    }
    if message.contains("NOT NULL constraint failed")
        || message.contains("CHECK constraint failed")
        || message.contains("violates not-null constraint")
        || message.contains("violates check constraint")
        || message.contains("invalid input syntax for")
    {
        let code = match operation {
            "exec" => "DB.EXEC_INVALID",
            "execTx" => "DB.EXEC_TX_INVALID",
            "queryOne" => "DB.QUERY_ONE_INVALID",
            _ => "DB.OPERATION_INVALID",
        };
        return (400, code, "validation");
    }
    if message.contains("UNIQUE constraint failed")
        || message.contains("duplicate key value violates unique constraint")
    {
        let code = match operation {
            "exec" => "DB.EXEC_CONFLICT",
            "execTx" => "DB.EXEC_TX_CONFLICT",
            "queryOne" => "DB.QUERY_ONE_CONFLICT",
            _ => "DB.OPERATION_CONFLICT",
        };
        return (409, code, "conflict");
    }
    let code = match operation {
        "exec" => "DB.EXEC_FAILED",
        "execTx" => "DB.EXEC_TX_FAILED",
        "queryOne" => "DB.QUERY_ONE_FAILED",
        _ => "DB.OPERATION_FAILED",
    };
    (500, code, "missing_dependency")
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
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return "0".to_string();
    }
    let parsed = match serde_json::from_str::<serde_json::Value>(trimmed) {
        Ok(parsed) => parsed,
        Err(_) => return trimmed.to_string(),
    };
    let canonical = canonicalize_lasm_db_params_value(parsed);
    serde_json::to_string(&canonical).unwrap_or_else(|_| trimmed.to_string())
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
            entry.insert(db);
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
    state.db_records_postgres_client.as_mut().ok_or_else(|| {
        "db adapter postgres requires SEC4_RT_LASM_DB_POSTGRES_DSN to be set".to_string()
    })
}

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
        client
            .prepare(query_template)
            .map_err(|err| format!("postgres prepare failed: {err}"))?
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
    let dsn = state.db_records_postgres_dsn.as_deref().ok_or_else(|| {
        "db adapter postgres requires SEC4_RT_LASM_DB_POSTGRES_DSN to be set".to_string()
    })?;
    let mut client = connect_lasm_dynamic_db_records_postgres(
        dsn,
        state.db_postgres_tls_mode,
        state.db_postgres_statement_timeout_ms.max(1),
        state.db_postgres_lock_timeout_ms.max(1),
        state.db_postgres_connect_timeout_ms.max(1),
    )?;
    ensure_lasm_dynamic_db_records_postgres_schema(&mut client)?;
    state.db_records_postgres_client = Some(client);
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
