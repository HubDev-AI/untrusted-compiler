use crate::lasm_db_adapter_state::LasmDbPostgresTlsMode;
use crate::lasm_db_config::LASM_DB_POSTGRES_DSN_CONFIG_ERROR_MESSAGE;
use crate::lasm_db_runtime_postgres::LasmPostgresThreadLocalConfig;
use crate::LasmDynamicResponseState;

#[inline(always)]
fn lasm_postgres_tls_mode_label(tls_mode: LasmDbPostgresTlsMode) -> &'static str {
    match tls_mode {
        LasmDbPostgresTlsMode::Auto => "auto",
        LasmDbPostgresTlsMode::Disable => "disable",
        LasmDbPostgresTlsMode::Require => "require",
    }
}

pub(crate) fn build_lasm_postgres_thread_local_config(
    state: &LasmDynamicResponseState,
) -> Result<LasmPostgresThreadLocalConfig, String> {
    let dsn = state
        .db_records_postgres_dsn
        .as_deref()
        .ok_or_else(|| LASM_DB_POSTGRES_DSN_CONFIG_ERROR_MESSAGE.to_string())?;
    let tls_mode = state.db_postgres_tls_mode;
    let tls_mode_label = lasm_postgres_tls_mode_label(tls_mode);
    let shared_client_pool_key = format!(
        "{tls_mode_label}\u{1f}{dsn}\u{1f}{}\u{1f}{}\u{1f}{}",
        state.db_postgres_statement_timeout_ms.max(1),
        state.db_postgres_lock_timeout_ms.max(1),
        state.db_postgres_connect_timeout_ms.max(1),
    );
    let schema_ensure_key = format!("{tls_mode_label}\u{1f}{dsn}");
    Ok(LasmPostgresThreadLocalConfig {
        dsn: dsn.to_string(),
        tls_mode,
        shared_client_pool_key,
        schema_ensure_key,
        statement_timeout_ms: state.db_postgres_statement_timeout_ms.max(1),
        lock_timeout_ms: state.db_postgres_lock_timeout_ms.max(1),
        connect_timeout_ms: state.db_postgres_connect_timeout_ms.max(1),
        db_postgres_statement_cache_max: state.db_postgres_statement_cache_max,
        db_postgres_placeholder_cache_max: state.db_postgres_placeholder_cache_max,
        retryable_conflict_retry_max: state.db_postgres_retryable_conflict_retry_max,
    })
}
