use crate::lasm_db_adapter_state::{
    connect_lasm_dynamic_db_records_postgres, ensure_lasm_dynamic_db_records_postgres_schema,
};
use crate::LasmDynamicResponseState;
use postgres::{Client as PostgresClient, Statement as PostgresStatement};

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
        "0".to_string()
    } else {
        trimmed.to_string()
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
    {
        return Ok(statement.clone());
    }
    let statement = {
        let client = lasm_dynamic_postgres_client_mut(state)?;
        client
            .prepare(query_template)
            .map_err(|err| format!("postgres prepare failed: {err}"))?
    };
    if state.db_records_postgres_statement_cache.len() >= state.db_postgres_statement_cache_max {
        let evicted = state.db_records_postgres_statement_cache.len() as u64;
        state.db_postgres_statement_cache_evictions_total = state
            .db_postgres_statement_cache_evictions_total
            .saturating_add(evicted);
        state.db_records_postgres_statement_cache.clear();
    }
    state
        .db_records_postgres_statement_cache
        .insert(query_template.to_string(), statement.clone());
    Ok(statement)
}

pub(crate) fn reconnect_lasm_dynamic_postgres_client(
    state: &mut LasmDynamicResponseState,
) -> Result<(), String> {
    let dsn = state.db_records_postgres_dsn.as_deref().ok_or_else(|| {
        "db adapter postgres requires SEC4_RT_LASM_DB_POSTGRES_DSN to be set".to_string()
    })?;
    let mut client = connect_lasm_dynamic_db_records_postgres(
        dsn,
        state.db_postgres_statement_timeout_ms.max(1),
        state.db_postgres_lock_timeout_ms.max(1),
        state.db_postgres_connect_timeout_ms.max(1),
    )?;
    ensure_lasm_dynamic_db_records_postgres_schema(&mut client)?;
    state.db_records_postgres_client = Some(client);
    state.db_records_postgres_statement_cache.clear();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::classify_lasm_db_runtime_error;

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
}
