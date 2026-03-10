use crate::lasm_db_adapter_state::{
    connect_lasm_dynamic_db_records_sqlite, load_lasm_dynamic_db_records_from_sqlite,
    normalize_lasm_db_sqlite_journal_mode, normalize_lasm_db_sqlite_synchronous,
    parse_lasm_db_postgres_tls_mode, resolve_lasm_db_sqlite_journal_mode,
    resolve_lasm_db_sqlite_synchronous, LasmDbPostgresTlsMode,
    LASM_DB_POSTGRES_CONNECT_TIMEOUT_MS_DEFAULT, LASM_DB_POSTGRES_LOCK_TIMEOUT_MS_DEFAULT,
    LASM_DB_POSTGRES_STATEMENT_TIMEOUT_MS_DEFAULT, LASM_DB_POSTGRES_TLS_MODE_ENV,
    LASM_DB_SQLITE_BUSY_TIMEOUT_MS_DEFAULT,
};
use crate::lasm_db_config::{
    resolve_lasm_dynamic_db_postgres_dsn, resolve_lasm_dynamic_db_records_adapter,
    resolve_lasm_dynamic_db_tx_max_handles, resolve_lasm_dynamic_store_base,
};
use crate::lasm_db_records_log::load_lasm_dynamic_db_records_from_disk;
use crate::lasm_db_runtime_postgres::{
    LasmPostgresThreadLocalClient, LasmPostgresThreadLocalConfig,
};
use postgres::{Client as PostgresClient, Statement as PostgresStatement};
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::env;
use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub(crate) enum LasmDbRecordsAdapter {
    #[default]
    RecordsLog,
    Sqlite,
    Postgres,
}

#[derive(Debug, Clone)]
pub(crate) struct LasmDbRecord {
    pub(crate) id: u64,
    pub(crate) op: String,
    pub(crate) db: i64,
    pub(crate) template: String,
    pub(crate) params: String,
    pub(crate) tx: i64,
    pub(crate) affected_rows: u64,
    pub(crate) created_at_ms: u64,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct LasmDbTxState {
    pub(crate) db: i64,
    pub(crate) active: bool,
    pub(crate) in_use: bool,
}

#[derive(Default)]
pub(crate) struct LasmDynamicResponseState {
    pub(crate) users_by_id: HashMap<String, serde_json::Value>,
    pub(crate) users_store_path: Option<PathBuf>,
    pub(crate) db_records: Vec<LasmDbRecord>,
    pub(crate) db_record_signatures: HashMap<String, usize>,
    pub(crate) db_latest_record_by_signature: HashMap<String, LasmDbRecord>,
    pub(crate) db_records_max: usize,
    pub(crate) db_records_dropped_total: u64,
    pub(crate) db_records_adapter: LasmDbRecordsAdapter,
    pub(crate) db_records_store_path: Option<PathBuf>,
    pub(crate) db_records_sqlite_store_path: Option<PathBuf>,
    pub(crate) db_records_sqlite_connection: Option<rusqlite::Connection>,
    pub(crate) db_records_postgres_dsn: Option<String>,
    pub(crate) db_records_postgres_client: Option<PostgresClient>,
    pub(crate) db_postgres_thread_local_config_cache: Option<LasmPostgresThreadLocalConfig>,
    pub(crate) db_postgres_tx_clients: HashMap<i64, LasmPostgresThreadLocalClient>,
    pub(crate) db_records_postgres_bootstrapped: bool,
    pub(crate) db_records_postgres_statement_cache: HashMap<String, PostgresStatement>,
    pub(crate) db_records_postgres_statement_cache_order: VecDeque<String>,
    pub(crate) db_postgres_placeholder_max_cache: HashMap<String, usize>,
    #[allow(dead_code)]
    pub(crate) db_postgres_placeholder_max_cache_order: VecDeque<String>,
    pub(crate) db_postgres_statement_cache_max: usize,
    pub(crate) db_postgres_placeholder_cache_max: usize,
    pub(crate) db_postgres_statement_cache_evictions_total: u64,
    pub(crate) db_postgres_placeholder_cache_evictions_total: u64,
    pub(crate) db_tx_handles: HashMap<i64, LasmDbTxState>,
    pub(crate) db_tx_max_handles: usize,
    pub(crate) db_postgres_statement_timeout_ms: u64,
    pub(crate) db_postgres_lock_timeout_ms: u64,
    pub(crate) db_postgres_connect_timeout_ms: u64,
    pub(crate) db_postgres_tls_mode: LasmDbPostgresTlsMode,
    pub(crate) db_postgres_retryable_conflict_retry_max: usize,
    pub(crate) db_postgres_retryable_conflict_retry_attempts_total: u64,
    pub(crate) db_postgres_retryable_conflict_retry_success_total: u64,
    pub(crate) db_postgres_stale_plan_reprepare_total: u64,
    pub(crate) db_sqlite_busy_timeout_ms: u64,
    pub(crate) db_sqlite_lock_retry_max: usize,
    pub(crate) db_sqlite_lock_retry_delay_ms: u64,
    pub(crate) db_sqlite_lock_retry_attempts_total: u64,
    pub(crate) db_sqlite_lock_retry_success_total: u64,
    pub(crate) db_sqlite_journal_mode: String,
    pub(crate) db_sqlite_synchronous: String,
    pub(crate) next_db_tx_handle: i64,
    pub(crate) next_db_record_id: u64,
}

pub(crate) const LASM_DYNAMIC_DB_POSTGRES_RECORDS_TABLE: &str = "sec4_lasm_db_records";
pub(crate) const LASM_DB_POSTGRES_STATEMENT_CACHE_MAX_DEFAULT: usize = 512;
pub(crate) const LASM_DB_POSTGRES_PLACEHOLDER_CACHE_MAX_DEFAULT: usize = 1024;
pub(crate) const LASM_DB_RECORDS_MAX_DEFAULT: usize = 10000;
pub(crate) const LASM_DB_POSTGRES_RETRYABLE_CONFLICT_RETRY_MAX_DEFAULT: usize = 1;
pub(crate) const LASM_DB_SQLITE_LOCK_RETRY_MAX_DEFAULT: usize = 2;
pub(crate) const LASM_DB_SQLITE_LOCK_RETRY_DELAY_MS_DEFAULT: u64 = 5;
pub(crate) const LASM_DB_POSTGRES_RECORD_ID_LOCAL_BITS: u32 = 32;
pub(crate) const LASM_DB_POSTGRES_RECORD_ID_LOCAL_MASK: u64 =
    (1u64 << LASM_DB_POSTGRES_RECORD_ID_LOCAL_BITS) - 1;

static LASM_DB_POSTGRES_RECORD_ID_NAMESPACE: OnceLock<u64> = OnceLock::new();

pub(crate) fn lasm_db_record_signature_key(db: i64, template: &str, params: &str) -> String {
    format!("{db}\u{1f}{template}\u{1f}{params}")
}

pub(crate) fn resolve_lasm_db_postgres_record_id_namespace() -> u64 {
    *LASM_DB_POSTGRES_RECORD_ID_NAMESPACE.get_or_init(|| {
        let pid = std::process::id() as u64;
        let now_nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos() as u64)
            .unwrap_or(pid);
        let mixed = now_nanos ^ now_nanos.rotate_left(17) ^ pid.rotate_left(7) ^ (pid << 32);
        (mixed & 0x7fff_ffff).max(1)
    })
}

pub(crate) fn next_lasm_db_record_counter(
    adapter: LasmDbRecordsAdapter,
    records: &[LasmDbRecord],
) -> u64 {
    match adapter {
        LasmDbRecordsAdapter::Postgres => {
            let namespace = resolve_lasm_db_postgres_record_id_namespace();
            records
                .iter()
                .filter_map(|record| {
                    let record_namespace = record.id >> LASM_DB_POSTGRES_RECORD_ID_LOCAL_BITS;
                    if record_namespace == namespace {
                        Some(record.id & LASM_DB_POSTGRES_RECORD_ID_LOCAL_MASK)
                    } else {
                        None
                    }
                })
                .max()
                .unwrap_or(0)
                .saturating_add(1)
        }
        _ => records
            .iter()
            .map(|record| record.id)
            .max()
            .unwrap_or(0)
            .saturating_add(1),
    }
}

pub(crate) fn compose_lasm_db_record_id(adapter: LasmDbRecordsAdapter, counter: u64) -> u64 {
    match adapter {
        LasmDbRecordsAdapter::Postgres => {
            let local_counter = counter.max(1).min(LASM_DB_POSTGRES_RECORD_ID_LOCAL_MASK);
            (resolve_lasm_db_postgres_record_id_namespace()
                << LASM_DB_POSTGRES_RECORD_ID_LOCAL_BITS)
                | local_counter
        }
        _ => counter,
    }
}

fn build_lasm_db_record_signature_counts(records: &[LasmDbRecord]) -> HashMap<String, usize> {
    let mut signatures = HashMap::new();
    for record in records {
        let signature = lasm_db_record_signature_key(record.db, &record.template, &record.params);
        *signatures.entry(signature).or_insert(0) += 1;
    }
    signatures
}

fn build_lasm_db_latest_record_by_signature(
    records: &[LasmDbRecord],
) -> HashMap<String, LasmDbRecord> {
    let mut latest = HashMap::new();
    for record in records {
        let signature = lasm_db_record_signature_key(record.db, &record.template, &record.params);
        latest.insert(signature, record.clone());
    }
    latest
}

fn decrement_lasm_db_record_signature(
    signatures: &mut HashMap<String, usize>,
    record: &LasmDbRecord,
) -> bool {
    let signature = lasm_db_record_signature_key(record.db, &record.template, &record.params);
    if let Some(count) = signatures.get_mut(signature.as_str()) {
        if *count <= 1 {
            signatures.remove(signature.as_str());
            return false;
        } else {
            *count -= 1;
            return true;
        }
    }
    false
}

fn truncate_lasm_db_records_to_capacity(
    records: &mut Vec<LasmDbRecord>,
    capacity: usize,
) -> Vec<LasmDbRecord> {
    let bounded_capacity = capacity.max(1);
    if records.len() > bounded_capacity {
        let overflow = records.len() - bounded_capacity;
        return records.drain(0..overflow).collect();
    }
    Vec::new()
}

pub(crate) fn append_lasm_dynamic_db_record(
    state: &mut LasmDynamicResponseState,
    record: LasmDbRecord,
) -> bool {
    let signature = lasm_db_record_signature_key(record.db, &record.template, &record.params);
    state.db_records.push(record);
    let appended_record = state
        .db_records
        .last()
        .cloned()
        .expect("append should retain pushed db record");
    *state.db_record_signatures.entry(signature).or_insert(0) += 1;
    state.db_latest_record_by_signature.insert(
        lasm_db_record_signature_key(
            appended_record.db,
            &appended_record.template,
            &appended_record.params,
        ),
        appended_record,
    );
    let dropped_records =
        truncate_lasm_db_records_to_capacity(&mut state.db_records, state.db_records_max);
    if !dropped_records.is_empty() {
        state.db_records_dropped_total = state
            .db_records_dropped_total
            .saturating_add(dropped_records.len() as u64);
        let mut signatures_needing_latest_refresh = Vec::new();
        for dropped_record in dropped_records.iter() {
            let dropped_signature = lasm_db_record_signature_key(
                dropped_record.db,
                &dropped_record.template,
                &dropped_record.params,
            );
            let signature_still_present =
                decrement_lasm_db_record_signature(&mut state.db_record_signatures, dropped_record);
            let latest_matches_dropped = state
                .db_latest_record_by_signature
                .get(dropped_signature.as_str())
                .map(|record| record.id == dropped_record.id)
                .unwrap_or(false);
            if latest_matches_dropped {
                if signature_still_present {
                    signatures_needing_latest_refresh.push(dropped_signature);
                } else {
                    state
                        .db_latest_record_by_signature
                        .remove(dropped_signature.as_str());
                }
            }
        }
        signatures_needing_latest_refresh.sort();
        signatures_needing_latest_refresh.dedup();
        for signature in signatures_needing_latest_refresh.into_iter() {
            let next_latest = state
                .db_records
                .iter()
                .rev()
                .find(|candidate| {
                    lasm_db_record_signature_key(
                        candidate.db,
                        &candidate.template,
                        &candidate.params,
                    ) == signature
                })
                .cloned();
            if let Some(record) = next_latest {
                state
                    .db_latest_record_by_signature
                    .insert(signature, record);
            } else {
                state
                    .db_latest_record_by_signature
                    .remove(signature.as_str());
            }
        }
        return true;
    }
    false
}

pub(crate) fn build_lasm_dynamic_response_state(
    project_path: Option<&Path>,
    explicit_db_base: Option<&Path>,
    explicit_db_records_adapter: Option<LasmDbRecordsAdapter>,
    explicit_db_postgres_dsn: Option<&str>,
    explicit_db_postgres_tls_mode: Option<LasmDbPostgresTlsMode>,
    explicit_db_postgres_statement_timeout_ms: Option<u64>,
    explicit_db_postgres_lock_timeout_ms: Option<u64>,
    explicit_db_postgres_connect_timeout_ms: Option<u64>,
    explicit_db_sqlite_busy_timeout_ms: Option<u64>,
    explicit_db_postgres_retryable_conflict_retry_max: Option<usize>,
    explicit_db_sqlite_lock_retry_max: Option<usize>,
    explicit_db_sqlite_lock_retry_delay_ms: Option<u64>,
    explicit_db_sqlite_journal_mode: Option<&str>,
    explicit_db_sqlite_synchronous: Option<&str>,
    explicit_db_tx_max_handles: Option<usize>,
    explicit_db_records_max: Option<usize>,
    explicit_db_postgres_statement_cache_max: Option<usize>,
    explicit_db_postgres_placeholder_cache_max: Option<usize>,
) -> Result<LasmDynamicResponseState, String> {
    let db_postgres_statement_timeout_ms = explicit_db_postgres_statement_timeout_ms
        .filter(|value| *value > 0)
        .unwrap_or_else(|| {
            resolve_lasm_env_positive_u64(
                "SEC4_RT_LASM_DB_POSTGRES_STATEMENT_TIMEOUT_MS",
                LASM_DB_POSTGRES_STATEMENT_TIMEOUT_MS_DEFAULT,
            )
        });
    let db_postgres_lock_timeout_ms = explicit_db_postgres_lock_timeout_ms
        .filter(|value| *value > 0)
        .unwrap_or_else(|| {
            resolve_lasm_env_positive_u64(
                "SEC4_RT_LASM_DB_POSTGRES_LOCK_TIMEOUT_MS",
                LASM_DB_POSTGRES_LOCK_TIMEOUT_MS_DEFAULT,
            )
        });
    let db_postgres_connect_timeout_ms = explicit_db_postgres_connect_timeout_ms
        .filter(|value| *value > 0)
        .unwrap_or_else(|| {
            resolve_lasm_env_positive_u64(
                "SEC4_RT_LASM_DB_POSTGRES_CONNECT_TIMEOUT_MS",
                LASM_DB_POSTGRES_CONNECT_TIMEOUT_MS_DEFAULT,
            )
        });
    let db_postgres_retryable_conflict_retry_max =
        explicit_db_postgres_retryable_conflict_retry_max.unwrap_or_else(|| {
            resolve_lasm_env_non_negative_usize(
                "SEC4_RT_LASM_DB_POSTGRES_RETRYABLE_CONFLICT_RETRY_MAX",
                LASM_DB_POSTGRES_RETRYABLE_CONFLICT_RETRY_MAX_DEFAULT,
            )
        });
    let db_sqlite_busy_timeout_ms = explicit_db_sqlite_busy_timeout_ms
        .filter(|value| *value > 0)
        .unwrap_or_else(|| {
            resolve_lasm_env_positive_u64(
                "SEC4_RT_LASM_SQLITE_BUSY_TIMEOUT_MS",
                LASM_DB_SQLITE_BUSY_TIMEOUT_MS_DEFAULT,
            )
        });
    let db_sqlite_lock_retry_max = explicit_db_sqlite_lock_retry_max.unwrap_or_else(|| {
        resolve_lasm_env_non_negative_usize(
            "SEC4_RT_LASM_DB_SQLITE_LOCK_RETRY_MAX",
            LASM_DB_SQLITE_LOCK_RETRY_MAX_DEFAULT,
        )
    });
    let db_sqlite_lock_retry_delay_ms =
        explicit_db_sqlite_lock_retry_delay_ms.unwrap_or_else(|| {
            resolve_lasm_env_non_negative_u64(
                "SEC4_RT_LASM_DB_SQLITE_LOCK_RETRY_DELAY_MS",
                LASM_DB_SQLITE_LOCK_RETRY_DELAY_MS_DEFAULT,
            )
        });
    let db_sqlite_journal_mode = if let Some(mode) = explicit_db_sqlite_journal_mode {
        normalize_lasm_db_sqlite_journal_mode(mode)
            .ok_or_else(|| format!("invalid --db-sqlite-journal-mode value `{mode}`"))?
            .to_string()
    } else {
        resolve_lasm_db_sqlite_journal_mode().to_string()
    };
    let db_sqlite_synchronous = if let Some(mode) = explicit_db_sqlite_synchronous {
        normalize_lasm_db_sqlite_synchronous(mode)
            .ok_or_else(|| format!("invalid --db-sqlite-synchronous value `{mode}`"))?
            .to_string()
    } else {
        resolve_lasm_db_sqlite_synchronous().to_string()
    };
    let db_postgres_tls_mode = resolve_lasm_db_postgres_tls_mode(explicit_db_postgres_tls_mode)?;
    let db_postgres_statement_cache_max = explicit_db_postgres_statement_cache_max
        .filter(|value| *value > 0)
        .unwrap_or_else(|| {
            resolve_lasm_env_positive_usize(
                "SEC4_RT_LASM_DB_POSTGRES_STATEMENT_CACHE_MAX",
                LASM_DB_POSTGRES_STATEMENT_CACHE_MAX_DEFAULT,
            )
        });
    let db_postgres_placeholder_cache_max = explicit_db_postgres_placeholder_cache_max
        .filter(|value| *value > 0)
        .unwrap_or_else(|| {
            resolve_lasm_env_positive_usize(
                "SEC4_RT_LASM_DB_POSTGRES_PLACEHOLDER_CACHE_MAX",
                LASM_DB_POSTGRES_PLACEHOLDER_CACHE_MAX_DEFAULT,
            )
        });
    let db_records_max = explicit_db_records_max
        .filter(|value| *value > 0)
        .unwrap_or_else(|| {
            resolve_lasm_env_positive_usize(
                "SEC4_RT_LASM_DB_RECORDS_MAX",
                LASM_DB_RECORDS_MAX_DEFAULT,
            )
        });
    let base = resolve_lasm_dynamic_store_base(project_path, explicit_db_base);
    let users_store_path = base.as_ref().map(|base| base.join("users.json"));
    let db_records_adapter = resolve_lasm_dynamic_db_records_adapter(explicit_db_records_adapter);
    let db_records_store_path = base.as_ref().map(|base| base.join("records.log"));
    let db_records_sqlite_store_path = base.as_ref().map(|base| base.join("records.sqlite3"));
    let db_records_postgres_dsn = resolve_lasm_dynamic_db_postgres_dsn(
        db_records_adapter,
        explicit_db_postgres_dsn,
        project_path,
    )?;
    if let Some(path) = users_store_path.as_ref() {
        if let Err(message) =
            ensure_lasm_dynamic_db_storage_path(path.as_path(), false, db_records_adapter)
        {
            eprintln!("warning: {message}");
        }
    }
    if let Some(path) = db_records_store_path.as_ref() {
        if let Err(message) =
            ensure_lasm_dynamic_db_storage_path(path.as_path(), true, db_records_adapter)
        {
            eprintln!("warning: {message}");
        }
    }
    if let Some(path) = db_records_sqlite_store_path.as_ref() {
        if let Err(message) =
            ensure_lasm_dynamic_db_storage_path(path.as_path(), false, db_records_adapter)
        {
            eprintln!("warning: {message}");
        }
    }
    let mut db_records_sqlite_connection = None;
    let db_records_postgres_client = None;
    let db_postgres_tx_clients = HashMap::new();
    let users_by_id = users_store_path
        .as_ref()
        .map(|path| load_lasm_dynamic_users_from_disk(path.as_path()))
        .unwrap_or_default();
    let mut db_records = match db_records_adapter {
        LasmDbRecordsAdapter::RecordsLog => db_records_store_path
            .as_ref()
            .map(|path| load_lasm_dynamic_db_records_from_disk(path.as_path()))
            .unwrap_or_default(),
        LasmDbRecordsAdapter::Sqlite => db_records_sqlite_store_path
            .as_ref()
            .map(|path| {
                let records = load_lasm_dynamic_db_records_from_sqlite(path.as_path());
                match connect_lasm_dynamic_db_records_sqlite(
                    path.as_path(),
                    db_sqlite_busy_timeout_ms,
                    db_sqlite_journal_mode.as_str(),
                    db_sqlite_synchronous.as_str(),
                ) {
                    Ok(connection) => {
                        db_records_sqlite_connection = Some(connection);
                    }
                    Err(err) => {
                        eprintln!(
                            "warning: LASM dynamic sqlite records connection bootstrap failed at `{}`: {err}",
                            path.display()
                        );
                    }
                }
                records
            })
            .unwrap_or_default(),
        LasmDbRecordsAdapter::Postgres => Vec::new(),
    };
    let startup_dropped = truncate_lasm_db_records_to_capacity(&mut db_records, db_records_max);
    let db_record_signatures = build_lasm_db_record_signature_counts(&db_records);
    let db_latest_record_by_signature = build_lasm_db_latest_record_by_signature(&db_records);
    let next_db_record_id = next_lasm_db_record_counter(db_records_adapter, &db_records);
    let db_tx_max_handles = resolve_lasm_dynamic_db_tx_max_handles(explicit_db_tx_max_handles)?;
    let db_tx_handles = HashMap::new();
    let db_records_postgres_statement_cache = HashMap::new();
    let db_records_postgres_statement_cache_order = VecDeque::new();
    let db_postgres_placeholder_max_cache = HashMap::new();
    let db_postgres_placeholder_max_cache_order = VecDeque::new();
    let next_db_tx_handle = 1;
    Ok(LasmDynamicResponseState {
        users_by_id,
        users_store_path,
        db_records,
        db_record_signatures,
        db_latest_record_by_signature,
        db_records_max,
        db_records_dropped_total: startup_dropped.len() as u64,
        db_records_adapter,
        db_records_store_path,
        db_records_sqlite_store_path,
        db_records_sqlite_connection,
        db_records_postgres_dsn,
        db_records_postgres_client,
        db_postgres_thread_local_config_cache: None,
        db_postgres_tx_clients,
        db_records_postgres_bootstrapped: false,
        db_records_postgres_statement_cache,
        db_records_postgres_statement_cache_order,
        db_postgres_placeholder_max_cache,
        db_postgres_placeholder_max_cache_order,
        db_postgres_statement_cache_max,
        db_postgres_placeholder_cache_max,
        db_postgres_statement_cache_evictions_total: 0,
        db_postgres_placeholder_cache_evictions_total: 0,
        db_tx_handles,
        db_tx_max_handles,
        db_postgres_statement_timeout_ms,
        db_postgres_lock_timeout_ms,
        db_postgres_connect_timeout_ms,
        db_postgres_tls_mode,
        db_postgres_retryable_conflict_retry_max,
        db_postgres_retryable_conflict_retry_attempts_total: 0,
        db_postgres_retryable_conflict_retry_success_total: 0,
        db_postgres_stale_plan_reprepare_total: 0,
        db_sqlite_busy_timeout_ms,
        db_sqlite_lock_retry_max,
        db_sqlite_lock_retry_delay_ms,
        db_sqlite_lock_retry_attempts_total: 0,
        db_sqlite_lock_retry_success_total: 0,
        db_sqlite_journal_mode,
        db_sqlite_synchronous,
        next_db_tx_handle,
        next_db_record_id,
    })
}

fn resolve_lasm_env_positive_u64(name: &str, default_value: u64) -> u64 {
    env::var(name)
        .ok()
        .and_then(|raw| raw.trim().parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(default_value)
}

fn resolve_lasm_env_positive_usize(name: &str, default_value: usize) -> usize {
    env::var(name)
        .ok()
        .and_then(|raw| raw.trim().parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(default_value)
}

fn resolve_lasm_env_non_negative_usize(name: &str, default_value: usize) -> usize {
    env::var(name)
        .ok()
        .and_then(|raw| raw.trim().parse::<usize>().ok())
        .unwrap_or(default_value)
}

fn resolve_lasm_env_non_negative_u64(name: &str, default_value: u64) -> u64 {
    env::var(name)
        .ok()
        .and_then(|raw| raw.trim().parse::<u64>().ok())
        .unwrap_or(default_value)
}

fn resolve_lasm_db_postgres_tls_mode(
    explicit_mode: Option<LasmDbPostgresTlsMode>,
) -> Result<LasmDbPostgresTlsMode, String> {
    if let Some(mode) = explicit_mode {
        return Ok(mode);
    }
    let Some(raw) = env::var(LASM_DB_POSTGRES_TLS_MODE_ENV).ok() else {
        return Ok(LasmDbPostgresTlsMode::Auto);
    };
    parse_lasm_db_postgres_tls_mode(raw.as_str()).ok_or_else(|| {
        format!(
            "invalid {LASM_DB_POSTGRES_TLS_MODE_ENV} value `{}`; expected one of: auto, disable, require",
            raw.trim()
        )
    })
}

fn load_lasm_dynamic_users_from_disk(path: &Path) -> HashMap<String, serde_json::Value> {
    let raw = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return HashMap::new(),
        Err(err) => {
            eprintln!(
                "warning: LASM dynamic users store load failed at `{}`: {err}",
                path.display()
            );
            return HashMap::new();
        }
    };
    let parsed: serde_json::Value = match serde_json::from_slice(&raw) {
        Ok(value) => value,
        Err(err) => {
            eprintln!(
                "warning: LASM dynamic users store parse failed at `{}`: {err}",
                path.display()
            );
            return HashMap::new();
        }
    };
    let Some(object) = parsed.as_object() else {
        eprintln!(
            "warning: LASM dynamic users store root must be a JSON object at `{}`",
            path.display()
        );
        return HashMap::new();
    };
    object
        .iter()
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect()
}

fn ensure_lasm_dynamic_db_storage_path(
    path: &Path,
    create_if_missing: bool,
    adapter: LasmDbRecordsAdapter,
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            format!(
                "could not create LASM dynamic storage directory `{}`: {err}",
                parent.display()
            )
        })?;
    }
    if !create_if_missing {
        return Ok(());
    }
    if adapter != LasmDbRecordsAdapter::RecordsLog {
        return Ok(());
    }
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|err| {
            format!(
                "could not create LASM dynamic records log file `{}`: {err}",
                path.display()
            )
        })?;
    Ok(())
}

pub(crate) fn persist_lasm_dynamic_users_to_disk(
    state: &LasmDynamicResponseState,
) -> Result<(), String> {
    let Some(path) = state.users_store_path.as_ref() else {
        return Ok(());
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            format!(
                "could not create LASM dynamic users store directory `{}`: {err}",
                parent.display()
            )
        })?;
    }
    let ordered = state
        .users_by_id
        .iter()
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect::<BTreeMap<_, _>>();
    let bytes = serde_json::to_vec_pretty(&ordered)
        .map_err(|err| format!("could not serialize LASM dynamic users store: {err}"))?;
    fs::write(path, bytes).map_err(|err| {
        format!(
            "could not write LASM dynamic users store `{}`: {err}",
            path.display()
        )
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        append_lasm_dynamic_db_record, lasm_db_record_signature_key, LasmDbRecord,
        LasmDbRecordsAdapter, LasmDynamicResponseState,
    };

    fn sample_record(id: u64) -> LasmDbRecord {
        LasmDbRecord {
            id,
            op: "exec".to_string(),
            db: 1,
            template: format!("SELECT {id}"),
            params: id.to_string(),
            tx: 0,
            affected_rows: 0,
            created_at_ms: 1,
        }
    }

    #[test]
    fn append_db_record_enforces_capacity_by_dropping_oldest() {
        let mut state = LasmDynamicResponseState {
            db_records_max: 2,
            db_records_adapter: LasmDbRecordsAdapter::RecordsLog,
            ..Default::default()
        };
        let first_append_overflowed = append_lasm_dynamic_db_record(&mut state, sample_record(1));
        let second_append_overflowed = append_lasm_dynamic_db_record(&mut state, sample_record(2));
        let third_append_overflowed = append_lasm_dynamic_db_record(&mut state, sample_record(3));
        assert!(!first_append_overflowed);
        assert!(!second_append_overflowed);
        assert!(third_append_overflowed);
        assert_eq!(state.db_records.len(), 2);
        assert_eq!(state.db_records[0].id, 2);
        assert_eq!(state.db_records[1].id, 3);
        assert_eq!(state.db_records_dropped_total, 1);
        assert!(!state
            .db_record_signatures
            .contains_key(lasm_db_record_signature_key(1, "SELECT 1", "1").as_str()));
        assert_eq!(state.db_record_signatures.len(), 2);
        assert_eq!(state.db_latest_record_by_signature.len(), 2);
    }

    #[test]
    fn append_db_record_keeps_signature_when_duplicate_survives_overflow() {
        let mut state = LasmDynamicResponseState {
            db_records_max: 2,
            db_records_adapter: LasmDbRecordsAdapter::RecordsLog,
            ..Default::default()
        };
        let mut first = sample_record(1);
        first.template = "SELECT 1".to_string();
        first.params = "same".to_string();
        let mut second = sample_record(2);
        second.template = "SELECT 1".to_string();
        second.params = "same".to_string();
        let mut third = sample_record(3);
        third.template = "SELECT 2".to_string();
        third.params = "other".to_string();
        append_lasm_dynamic_db_record(&mut state, first);
        append_lasm_dynamic_db_record(&mut state, second);
        let overflowed = append_lasm_dynamic_db_record(&mut state, third);
        assert!(overflowed);
        let duplicated_signature = lasm_db_record_signature_key(1, "SELECT 1", "same");
        assert!(state
            .db_record_signatures
            .contains_key(duplicated_signature.as_str()));
        assert_eq!(
            state
                .db_record_signatures
                .get(duplicated_signature.as_str())
                .copied(),
            Some(1)
        );
        let latest_signature_record = state
            .db_latest_record_by_signature
            .get(duplicated_signature.as_str())
            .expect("latest signature map should retain surviving duplicate");
        assert_eq!(latest_signature_record.id, 2);
    }
}
