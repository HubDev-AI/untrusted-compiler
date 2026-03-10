use crate::lasm_db_adapter_state::LasmDbPostgresTlsMode;
use crate::lasm_db_config::LASM_DB_POSTGRES_DSN_CONFIG_ERROR_MESSAGE;
use crate::lasm_db_runtime_postgres::LasmPostgresThreadLocalConfig;
use crate::{LasmDynamicResponseState, LASM_INTERNAL_DB_OP_SEQUENCE_MAX};
use std::env;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;

const LASM_DB_OP_SEQUENCE_MAX_ENV: &str = "SEC4_RT_LASM_DB_OP_SEQUENCE_MAX";
const LASM_DB_OP_SEQUENCE_MAX_DEFAULT: usize = LASM_INTERNAL_DB_OP_SEQUENCE_MAX;
const LASM_DB_OP_SEQUENCE_MAX_MIN: usize = 2;
const LASM_DB_OP_SEQUENCE_MAX_MAX: usize = 4096;
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
fn lasm_postgres_tls_mode_label(tls_mode: LasmDbPostgresTlsMode) -> &'static str {
    match tls_mode {
        LasmDbPostgresTlsMode::Auto => "auto",
        LasmDbPostgresTlsMode::Disable => "disable",
        LasmDbPostgresTlsMode::Require => "require",
    }
}

pub(crate) fn build_lasm_postgres_thread_local_config(
    state: &mut LasmDynamicResponseState,
) -> Result<LasmPostgresThreadLocalConfig, String> {
    let dsn = state
        .db_records_postgres_dsn
        .as_deref()
        .ok_or_else(|| LASM_DB_POSTGRES_DSN_CONFIG_ERROR_MESSAGE.to_string())?;
    let tls_mode = state.db_postgres_tls_mode;
    let statement_timeout_ms = state.db_postgres_statement_timeout_ms.max(1);
    let lock_timeout_ms = state.db_postgres_lock_timeout_ms.max(1);
    let connect_timeout_ms = state.db_postgres_connect_timeout_ms.max(1);
    let db_postgres_statement_cache_max = state.db_postgres_statement_cache_max;
    let db_postgres_placeholder_cache_max = state.db_postgres_placeholder_cache_max;
    let retryable_conflict_retry_max = state.db_postgres_retryable_conflict_retry_max;
    if let Some(cached) = state.db_postgres_thread_local_config_cache.as_ref() {
        if cached.dsn == dsn
            && cached.tls_mode == tls_mode
            && cached.statement_timeout_ms == statement_timeout_ms
            && cached.lock_timeout_ms == lock_timeout_ms
            && cached.connect_timeout_ms == connect_timeout_ms
            && cached.db_postgres_statement_cache_max == db_postgres_statement_cache_max
            && cached.db_postgres_placeholder_cache_max == db_postgres_placeholder_cache_max
            && cached.retryable_conflict_retry_max == retryable_conflict_retry_max
        {
            return Ok(cached.clone());
        }
    }
    let tls_mode_label = lasm_postgres_tls_mode_label(tls_mode);
    let shared_client_pool_key = format!(
        "{tls_mode_label}\u{1f}{dsn}\u{1f}{}\u{1f}{}\u{1f}{}",
        statement_timeout_ms, lock_timeout_ms, connect_timeout_ms,
    );
    let schema_ensure_key = format!("{tls_mode_label}\u{1f}{dsn}");
    let config = LasmPostgresThreadLocalConfig {
        dsn: dsn.to_string(),
        tls_mode,
        shared_client_pool_key,
        schema_ensure_key,
        statement_timeout_ms,
        lock_timeout_ms,
        connect_timeout_ms,
        db_postgres_statement_cache_max,
        db_postgres_placeholder_cache_max,
        retryable_conflict_retry_max,
    };
    state.db_postgres_thread_local_config_cache = Some(config.clone());
    Ok(config)
}

#[inline(always)]
pub(crate) fn resolve_lasm_db_sql_template_max_bytes() -> usize {
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

#[inline(always)]
pub(crate) fn set_lasm_db_sql_template_max_bytes_override(value: Option<usize>) {
    LASM_DB_SQL_TEMPLATE_MAX_BYTES_OVERRIDE.store(value.unwrap_or(0), Ordering::Relaxed);
}

#[inline(always)]
pub(crate) fn resolve_lasm_db_params_max_bytes() -> usize {
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

#[inline(always)]
pub(crate) fn set_lasm_db_params_max_bytes_override(value: Option<usize>) {
    LASM_DB_PARAMS_MAX_BYTES_OVERRIDE.store(value.unwrap_or(0), Ordering::Relaxed);
}

#[inline(always)]
pub(crate) fn resolve_lasm_db_params_max_entries() -> usize {
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

#[inline(always)]
pub(crate) fn set_lasm_db_params_max_entries_override(value: Option<usize>) {
    LASM_DB_PARAMS_MAX_ENTRIES_OVERRIDE.store(value.unwrap_or(0), Ordering::Relaxed);
}

#[inline(always)]
pub(crate) fn resolve_lasm_db_op_sequence_max() -> usize {
    let override_value = LASM_DB_OP_SEQUENCE_MAX_OVERRIDE.load(Ordering::Relaxed);
    if override_value != 0 {
        return override_value.clamp(LASM_DB_OP_SEQUENCE_MAX_MIN, LASM_DB_OP_SEQUENCE_MAX_MAX);
    }
    *LASM_DB_OP_SEQUENCE_MAX_RESOLVED.get_or_init(|| {
        env::var(LASM_DB_OP_SEQUENCE_MAX_ENV)
            .ok()
            .and_then(|raw| raw.trim().parse::<usize>().ok())
            .map(|value| value.clamp(LASM_DB_OP_SEQUENCE_MAX_MIN, LASM_DB_OP_SEQUENCE_MAX_MAX))
            .unwrap_or(LASM_DB_OP_SEQUENCE_MAX_DEFAULT)
    })
}

#[inline(always)]
pub(crate) fn set_lasm_db_op_sequence_max_override(value: Option<usize>) {
    LASM_DB_OP_SEQUENCE_MAX_OVERRIDE.store(value.unwrap_or(0), Ordering::Relaxed);
}

#[inline(always)]
pub(crate) fn resolve_lasm_db_query_one_row_max_bytes() -> usize {
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

#[inline(always)]
pub(crate) fn set_lasm_db_query_one_row_max_bytes_override(value: Option<usize>) {
    LASM_DB_QUERY_ONE_ROW_MAX_BYTES_OVERRIDE.store(value.unwrap_or(0), Ordering::Relaxed);
}

#[inline(always)]
pub(crate) fn resolve_lasm_db_query_one_row_max_columns() -> usize {
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

#[inline(always)]
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
