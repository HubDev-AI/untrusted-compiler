use crate::lasm_db_adapter_state::{
    connect_lasm_dynamic_db_records_postgres, connect_lasm_dynamic_db_records_sqlite,
    ensure_lasm_dynamic_db_records_postgres_schema, load_lasm_dynamic_db_records_from_postgres,
    load_lasm_dynamic_db_records_from_sqlite, parse_lasm_db_postgres_tls_mode,
    LasmDbPostgresTlsMode, LASM_DB_POSTGRES_CONNECT_TIMEOUT_MS_DEFAULT,
    LASM_DB_POSTGRES_LOCK_TIMEOUT_MS_DEFAULT, LASM_DB_POSTGRES_STATEMENT_TIMEOUT_MS_DEFAULT,
    LASM_DB_POSTGRES_TLS_MODE_ENV, LASM_DB_SQLITE_BUSY_TIMEOUT_MS_DEFAULT,
};
use crate::lasm_db_config::{
    resolve_lasm_dynamic_db_postgres_dsn, resolve_lasm_dynamic_db_records_adapter,
    resolve_lasm_dynamic_db_tx_max_handles, resolve_lasm_dynamic_store_base,
};
use crate::lasm_db_records_log::load_lasm_dynamic_db_records_from_disk;
use postgres::{Client as PostgresClient, Statement as PostgresStatement};
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

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

#[derive(Default)]
pub(crate) struct LasmDynamicResponseState {
    pub(crate) users_by_id: HashMap<String, serde_json::Value>,
    pub(crate) users_store_path: Option<PathBuf>,
    pub(crate) db_records: Vec<LasmDbRecord>,
    pub(crate) db_record_signatures: HashMap<String, usize>,
    pub(crate) db_records_max: usize,
    pub(crate) db_records_dropped_total: u64,
    pub(crate) db_records_adapter: LasmDbRecordsAdapter,
    pub(crate) db_records_store_path: Option<PathBuf>,
    pub(crate) db_records_sqlite_store_path: Option<PathBuf>,
    pub(crate) db_records_sqlite_connection: Option<rusqlite::Connection>,
    pub(crate) db_records_postgres_dsn: Option<String>,
    pub(crate) db_records_postgres_client: Option<PostgresClient>,
    pub(crate) db_records_postgres_statement_cache: HashMap<String, PostgresStatement>,
    pub(crate) db_records_postgres_statement_cache_order: VecDeque<String>,
    pub(crate) db_postgres_placeholder_max_cache: HashMap<String, usize>,
    pub(crate) db_postgres_placeholder_max_cache_order: VecDeque<String>,
    pub(crate) db_postgres_statement_cache_max: usize,
    pub(crate) db_postgres_placeholder_cache_max: usize,
    pub(crate) db_postgres_statement_cache_evictions_total: u64,
    pub(crate) db_postgres_placeholder_cache_evictions_total: u64,
    pub(crate) db_tx_handles: HashMap<i64, i64>,
    pub(crate) db_tx_max_handles: usize,
    pub(crate) db_postgres_statement_timeout_ms: u64,
    pub(crate) db_postgres_lock_timeout_ms: u64,
    pub(crate) db_postgres_connect_timeout_ms: u64,
    pub(crate) db_postgres_tls_mode: LasmDbPostgresTlsMode,
    pub(crate) db_sqlite_busy_timeout_ms: u64,
    pub(crate) next_db_tx_handle: i64,
    pub(crate) next_db_record_id: u64,
}

pub(crate) const LASM_DYNAMIC_DB_POSTGRES_RECORDS_TABLE: &str = "sec4_lasm_db_records";
pub(crate) const LASM_DB_POSTGRES_STATEMENT_CACHE_MAX_DEFAULT: usize = 512;
pub(crate) const LASM_DB_POSTGRES_PLACEHOLDER_CACHE_MAX_DEFAULT: usize = 1024;
pub(crate) const LASM_DB_RECORDS_MAX_DEFAULT: usize = 10000;

pub(crate) fn lasm_db_record_signature_key(db: i64, template: &str, params: &str) -> String {
    format!("{db}\u{1f}{template}\u{1f}{params}")
}

fn build_lasm_db_record_signature_counts(records: &[LasmDbRecord]) -> HashMap<String, usize> {
    let mut signatures = HashMap::new();
    for record in records {
        let signature = lasm_db_record_signature_key(record.db, &record.template, &record.params);
        *signatures.entry(signature).or_insert(0) += 1;
    }
    signatures
}

fn decrement_lasm_db_record_signature(
    signatures: &mut HashMap<String, usize>,
    record: &LasmDbRecord,
) {
    let signature = lasm_db_record_signature_key(record.db, &record.template, &record.params);
    if let Some(count) = signatures.get_mut(signature.as_str()) {
        if *count <= 1 {
            signatures.remove(signature.as_str());
        } else {
            *count -= 1;
        }
    }
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
    *state.db_record_signatures.entry(signature).or_insert(0) += 1;
    let dropped_records =
        truncate_lasm_db_records_to_capacity(&mut state.db_records, state.db_records_max);
    if !dropped_records.is_empty() {
        state.db_records_dropped_total = state
            .db_records_dropped_total
            .saturating_add(dropped_records.len() as u64);
        for dropped_record in dropped_records.iter() {
            decrement_lasm_db_record_signature(&mut state.db_record_signatures, dropped_record);
        }
        return true;
    }
    false
}

pub(crate) fn build_lasm_dynamic_response_state(
    explicit_db_base: Option<&Path>,
    explicit_db_records_adapter: Option<LasmDbRecordsAdapter>,
    explicit_db_postgres_dsn: Option<&str>,
    explicit_db_postgres_tls_mode: Option<LasmDbPostgresTlsMode>,
    explicit_db_postgres_statement_timeout_ms: Option<u64>,
    explicit_db_postgres_lock_timeout_ms: Option<u64>,
    explicit_db_postgres_connect_timeout_ms: Option<u64>,
    explicit_db_sqlite_busy_timeout_ms: Option<u64>,
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
    let db_sqlite_busy_timeout_ms = explicit_db_sqlite_busy_timeout_ms
        .filter(|value| *value > 0)
        .unwrap_or_else(|| {
            resolve_lasm_env_positive_u64(
                "SEC4_RT_LASM_SQLITE_BUSY_TIMEOUT_MS",
                LASM_DB_SQLITE_BUSY_TIMEOUT_MS_DEFAULT,
            )
        });
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
    let base = resolve_lasm_dynamic_store_base(explicit_db_base);
    let users_store_path = base.as_ref().map(|base| base.join("users.json"));
    let db_records_adapter = resolve_lasm_dynamic_db_records_adapter(explicit_db_records_adapter);
    let db_records_store_path = base.as_ref().map(|base| base.join("records.log"));
    let db_records_sqlite_store_path = base.as_ref().map(|base| base.join("records.sqlite3"));
    let db_records_postgres_dsn =
        resolve_lasm_dynamic_db_postgres_dsn(db_records_adapter, explicit_db_postgres_dsn)?;
    let mut db_records_sqlite_connection = None;
    let mut db_records_postgres_client = None;
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
                match connect_lasm_dynamic_db_records_sqlite(path.as_path(), db_sqlite_busy_timeout_ms)
                {
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
        LasmDbRecordsAdapter::Postgres => {
            let dsn = db_records_postgres_dsn
                .as_ref()
                .ok_or_else(|| {
                    "db adapter postgres requires SEC4_RT_LASM_DB_POSTGRES_DSN to be set"
                        .to_string()
                })?
                .as_str();
            let mut client = connect_lasm_dynamic_db_records_postgres(
                dsn,
                db_postgres_tls_mode,
                db_postgres_statement_timeout_ms,
                db_postgres_lock_timeout_ms,
                db_postgres_connect_timeout_ms,
            )?;
            ensure_lasm_dynamic_db_records_postgres_schema(&mut client)?;
            let records = load_lasm_dynamic_db_records_from_postgres(&mut client)?;
            db_records_postgres_client = Some(client);
            records
        }
    };
    let startup_dropped = truncate_lasm_db_records_to_capacity(&mut db_records, db_records_max);
    let db_record_signatures = build_lasm_db_record_signature_counts(&db_records);
    let next_db_record_id = db_records
        .iter()
        .map(|record| record.id)
        .max()
        .unwrap_or(0)
        .saturating_add(1);
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
        db_records_max,
        db_records_dropped_total: startup_dropped.len() as u64,
        db_records_adapter,
        db_records_store_path,
        db_records_sqlite_store_path,
        db_records_sqlite_connection,
        db_records_postgres_dsn,
        db_records_postgres_client,
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
        db_sqlite_busy_timeout_ms,
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
    }
}
