use crate::lasm_db_config::LASM_DB_POSTGRES_DSN_CONFIG_ERROR_MESSAGE;
use crate::lasm_db_runtime_postgres::LasmPostgresThreadLocalConfig;
use crate::LasmDynamicResponseState;

pub(crate) fn build_lasm_postgres_thread_local_config(
    state: &LasmDynamicResponseState,
) -> Result<LasmPostgresThreadLocalConfig, String> {
    let dsn = state
        .db_records_postgres_dsn
        .as_deref()
        .ok_or_else(|| LASM_DB_POSTGRES_DSN_CONFIG_ERROR_MESSAGE.to_string())?;
    Ok(LasmPostgresThreadLocalConfig {
        dsn: dsn.to_string(),
        tls_mode: state.db_postgres_tls_mode,
        statement_timeout_ms: state.db_postgres_statement_timeout_ms.max(1),
        lock_timeout_ms: state.db_postgres_lock_timeout_ms.max(1),
        connect_timeout_ms: state.db_postgres_connect_timeout_ms.max(1),
        db_postgres_statement_cache_max: state.db_postgres_statement_cache_max,
        db_postgres_placeholder_cache_max: state.db_postgres_placeholder_cache_max,
        retryable_conflict_retry_max: state.db_postgres_retryable_conflict_retry_max,
    })
}
