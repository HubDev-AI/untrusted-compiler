use std::env;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;

const LASM_DB_OP_SEQUENCE_MAX_ENV: &str = "SEC4_RT_LASM_DB_OP_SEQUENCE_MAX";
const LASM_DB_OP_SEQUENCE_MAX_DEFAULT: usize = 64;
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
static LASM_DB_RECORDS_PERSIST_ENABLED: OnceLock<bool> = OnceLock::new();
static LASM_DB_RECORDS_CAPTURE_ENABLED: OnceLock<bool> = OnceLock::new();

static LASM_DB_SQL_TEMPLATE_MAX_BYTES_OVERRIDE: AtomicUsize = AtomicUsize::new(0);
static LASM_DB_PARAMS_MAX_BYTES_OVERRIDE: AtomicUsize = AtomicUsize::new(0);
static LASM_DB_PARAMS_MAX_ENTRIES_OVERRIDE: AtomicUsize = AtomicUsize::new(0);
static LASM_DB_OP_SEQUENCE_MAX_OVERRIDE: AtomicUsize = AtomicUsize::new(0);
static LASM_DB_QUERY_ONE_ROW_MAX_BYTES_OVERRIDE: AtomicUsize = AtomicUsize::new(0);
static LASM_DB_QUERY_ONE_ROW_MAX_COLUMNS_OVERRIDE: AtomicUsize = AtomicUsize::new(0);

const LASM_DB_RECORDS_PERSIST_ENABLED_ENV_KEYS: [&str; 2] = [
    "SEC4_DB_ALPHA_DB_RECORDS_PERSIST_ENABLED",
    "SEC4_RT_LASM_DB_RECORDS_PERSIST_ENABLED",
];
const LASM_DB_RECORDS_CAPTURE_ENABLED_ENV_KEYS: [&str; 2] = [
    "SEC4_DB_ALPHA_DB_RECORDS_CAPTURE_ENABLED",
    "SEC4_RT_LASM_DB_RECORDS_CAPTURE_ENABLED",
];

#[derive(Debug, Clone, Copy, Default)]
pub struct LasmDbRuntimeLimitOverrides {
    pub query_one_row_max_bytes: Option<usize>,
    pub query_one_row_max_columns: Option<usize>,
    pub sql_template_max_bytes: Option<usize>,
    pub params_max_bytes: Option<usize>,
    pub params_max_entries: Option<usize>,
    pub op_sequence_max: Option<usize>,
}

#[inline(always)]
pub fn resolve_lasm_db_sql_template_max_bytes() -> usize {
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
pub fn set_lasm_db_sql_template_max_bytes_override(value: Option<usize>) {
    LASM_DB_SQL_TEMPLATE_MAX_BYTES_OVERRIDE.store(value.unwrap_or(0), Ordering::Relaxed);
}

#[inline(always)]
pub fn resolve_lasm_db_params_max_bytes() -> usize {
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
pub fn set_lasm_db_params_max_bytes_override(value: Option<usize>) {
    LASM_DB_PARAMS_MAX_BYTES_OVERRIDE.store(value.unwrap_or(0), Ordering::Relaxed);
}

#[inline(always)]
pub fn resolve_lasm_db_params_max_entries() -> usize {
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
pub fn set_lasm_db_params_max_entries_override(value: Option<usize>) {
    LASM_DB_PARAMS_MAX_ENTRIES_OVERRIDE.store(value.unwrap_or(0), Ordering::Relaxed);
}

#[inline(always)]
pub fn resolve_lasm_db_op_sequence_max() -> usize {
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
pub fn set_lasm_db_op_sequence_max_override(value: Option<usize>) {
    LASM_DB_OP_SEQUENCE_MAX_OVERRIDE.store(value.unwrap_or(0), Ordering::Relaxed);
}

#[inline(always)]
pub fn resolve_lasm_db_query_one_row_max_bytes() -> usize {
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
pub fn set_lasm_db_query_one_row_max_bytes_override(value: Option<usize>) {
    LASM_DB_QUERY_ONE_ROW_MAX_BYTES_OVERRIDE.store(value.unwrap_or(0), Ordering::Relaxed);
}

#[inline(always)]
pub fn resolve_lasm_db_query_one_row_max_columns() -> usize {
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
pub fn set_lasm_db_query_one_row_max_columns_override(value: Option<usize>) {
    LASM_DB_QUERY_ONE_ROW_MAX_COLUMNS_OVERRIDE.store(value.unwrap_or(0), Ordering::Relaxed);
}

pub fn apply_lasm_db_runtime_limit_overrides(overrides: LasmDbRuntimeLimitOverrides) {
    set_lasm_db_query_one_row_max_bytes_override(overrides.query_one_row_max_bytes);
    set_lasm_db_query_one_row_max_columns_override(overrides.query_one_row_max_columns);
    set_lasm_db_sql_template_max_bytes_override(overrides.sql_template_max_bytes);
    set_lasm_db_params_max_bytes_override(overrides.params_max_bytes);
    set_lasm_db_params_max_entries_override(overrides.params_max_entries);
    set_lasm_db_op_sequence_max_override(overrides.op_sequence_max);
}

pub fn lasm_db_records_persist_enabled() -> bool {
    *LASM_DB_RECORDS_PERSIST_ENABLED.get_or_init(|| {
        for key in LASM_DB_RECORDS_PERSIST_ENABLED_ENV_KEYS {
            let Ok(raw) = env::var(key) else {
                continue;
            };
            let normalized = raw.trim().to_ascii_lowercase();
            if normalized.is_empty() {
                continue;
            }
            return !matches!(normalized.as_str(), "0" | "false" | "no" | "off");
        }
        true
    })
}

pub fn lasm_db_records_capture_enabled() -> bool {
    *LASM_DB_RECORDS_CAPTURE_ENABLED.get_or_init(|| {
        for key in LASM_DB_RECORDS_CAPTURE_ENABLED_ENV_KEYS {
            let Ok(raw) = env::var(key) else {
                continue;
            };
            let normalized = raw.trim().to_ascii_lowercase();
            if normalized.is_empty() {
                continue;
            }
            return !matches!(normalized.as_str(), "0" | "false" | "no" | "off");
        }
        true
    })
}
