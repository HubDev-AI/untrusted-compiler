use crate::lasm_db_adapter_state::{
    connect_lasm_dynamic_db_records_postgres, ensure_lasm_dynamic_db_records_postgres_schema,
};
use crate::LasmDynamicResponseState;
use postgres::Client as PostgresClient;

pub(crate) fn classify_lasm_db_runtime_error(
    operation: &str,
    message: &str,
) -> (u16, &'static str, &'static str) {
    if message.contains("requires SEC4_RT_LASM_DB_POSTGRES_DSN")
        || message.contains("sqlite records store path unavailable")
    {
        return (500, "DB.ADAPTER_CONFIG_INVALID", "internal");
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

pub(crate) fn allocate_lasm_db_tx_handle(
    state: &mut LasmDynamicResponseState,
    db: i64,
) -> Option<i64> {
    if state.db_tx_handles.len() >= state.db_tx_max_handles {
        return None;
    }
    let tx = state.next_db_tx_handle.max(1);
    state.next_db_tx_handle = tx.saturating_add(1).max(1);
    state.db_tx_handles.insert(tx, db);
    Some(tx)
}

pub(crate) fn lasm_dynamic_postgres_client_mut(
    state: &mut LasmDynamicResponseState,
) -> Result<&mut PostgresClient, String> {
    state.db_records_postgres_client.as_mut().ok_or_else(|| {
        "db adapter postgres requires SEC4_RT_LASM_DB_POSTGRES_DSN to be set".to_string()
    })
}

pub(crate) fn reconnect_lasm_dynamic_postgres_client(
    state: &mut LasmDynamicResponseState,
) -> Result<(), String> {
    let dsn = state.db_records_postgres_dsn.as_deref().ok_or_else(|| {
        "db adapter postgres requires SEC4_RT_LASM_DB_POSTGRES_DSN to be set".to_string()
    })?;
    let mut client = connect_lasm_dynamic_db_records_postgres(dsn)?;
    ensure_lasm_dynamic_db_records_postgres_schema(&mut client)?;
    state.db_records_postgres_client = Some(client);
    Ok(())
}
