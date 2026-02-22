use crate::lasm_db_adapter_state::{
    connect_lasm_dynamic_db_records_postgres, connect_lasm_dynamic_db_records_sqlite,
    ensure_lasm_dynamic_db_records_postgres_schema, load_lasm_dynamic_db_records_from_postgres,
    load_lasm_dynamic_db_records_from_sqlite, LASM_DB_POSTGRES_CONNECT_TIMEOUT_MS_DEFAULT,
    LASM_DB_POSTGRES_LOCK_TIMEOUT_MS_DEFAULT, LASM_DB_POSTGRES_STATEMENT_TIMEOUT_MS_DEFAULT,
    LASM_DB_SQLITE_BUSY_TIMEOUT_MS_DEFAULT,
};
use crate::lasm_db_config::{
    resolve_lasm_dynamic_db_postgres_dsn, resolve_lasm_dynamic_db_records_adapter,
    resolve_lasm_dynamic_db_tx_max_handles, resolve_lasm_dynamic_store_base,
};
use crate::lasm_db_records_log::load_lasm_dynamic_db_records_from_disk;
use postgres::{Client as PostgresClient, Statement as PostgresStatement};
use std::collections::{BTreeMap, HashMap};
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
    pub(crate) db_records_adapter: LasmDbRecordsAdapter,
    pub(crate) db_records_store_path: Option<PathBuf>,
    pub(crate) db_records_sqlite_store_path: Option<PathBuf>,
    pub(crate) db_records_sqlite_connection: Option<rusqlite::Connection>,
    pub(crate) db_records_postgres_dsn: Option<String>,
    pub(crate) db_records_postgres_client: Option<PostgresClient>,
    pub(crate) db_records_postgres_statement_cache: HashMap<String, PostgresStatement>,
    pub(crate) db_postgres_placeholder_max_cache: HashMap<String, usize>,
    pub(crate) db_postgres_statement_cache_max: usize,
    pub(crate) db_postgres_placeholder_cache_max: usize,
    pub(crate) db_tx_handles: HashMap<i64, i64>,
    pub(crate) db_tx_max_handles: usize,
    pub(crate) db_postgres_statement_timeout_ms: u64,
    pub(crate) db_postgres_lock_timeout_ms: u64,
    pub(crate) db_postgres_connect_timeout_ms: u64,
    pub(crate) db_sqlite_busy_timeout_ms: u64,
    pub(crate) next_db_tx_handle: i64,
    pub(crate) next_db_record_id: u64,
}

pub(crate) const LASM_DYNAMIC_DB_POSTGRES_RECORDS_TABLE: &str = "sec4_lasm_db_records";
pub(crate) const LASM_DB_POSTGRES_STATEMENT_CACHE_MAX_DEFAULT: usize = 512;
pub(crate) const LASM_DB_POSTGRES_PLACEHOLDER_CACHE_MAX_DEFAULT: usize = 1024;

pub(crate) fn build_lasm_dynamic_response_state(
    explicit_db_base: Option<&Path>,
    explicit_db_records_adapter: Option<LasmDbRecordsAdapter>,
    explicit_db_postgres_dsn: Option<&str>,
    explicit_db_tx_max_handles: Option<usize>,
) -> Result<LasmDynamicResponseState, String> {
    let db_postgres_statement_timeout_ms = resolve_lasm_env_positive_u64(
        "SEC4_RT_LASM_DB_POSTGRES_STATEMENT_TIMEOUT_MS",
        LASM_DB_POSTGRES_STATEMENT_TIMEOUT_MS_DEFAULT,
    );
    let db_postgres_lock_timeout_ms = resolve_lasm_env_positive_u64(
        "SEC4_RT_LASM_DB_POSTGRES_LOCK_TIMEOUT_MS",
        LASM_DB_POSTGRES_LOCK_TIMEOUT_MS_DEFAULT,
    );
    let db_postgres_connect_timeout_ms = resolve_lasm_env_positive_u64(
        "SEC4_RT_LASM_DB_POSTGRES_CONNECT_TIMEOUT_MS",
        LASM_DB_POSTGRES_CONNECT_TIMEOUT_MS_DEFAULT,
    );
    let db_sqlite_busy_timeout_ms = resolve_lasm_env_positive_u64(
        "SEC4_RT_LASM_SQLITE_BUSY_TIMEOUT_MS",
        LASM_DB_SQLITE_BUSY_TIMEOUT_MS_DEFAULT,
    );
    let db_postgres_statement_cache_max = resolve_lasm_env_positive_usize(
        "SEC4_RT_LASM_DB_POSTGRES_STATEMENT_CACHE_MAX",
        LASM_DB_POSTGRES_STATEMENT_CACHE_MAX_DEFAULT,
    );
    let db_postgres_placeholder_cache_max = resolve_lasm_env_positive_usize(
        "SEC4_RT_LASM_DB_POSTGRES_PLACEHOLDER_CACHE_MAX",
        LASM_DB_POSTGRES_PLACEHOLDER_CACHE_MAX_DEFAULT,
    );
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
    let db_records = match db_records_adapter {
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
    let next_db_record_id = db_records
        .iter()
        .map(|record| record.id)
        .max()
        .unwrap_or(0)
        .saturating_add(1);
    let db_tx_max_handles = resolve_lasm_dynamic_db_tx_max_handles(explicit_db_tx_max_handles)?;
    let db_tx_handles = HashMap::new();
    let db_records_postgres_statement_cache = HashMap::new();
    let db_postgres_placeholder_max_cache = HashMap::new();
    let next_db_tx_handle = 1;
    Ok(LasmDynamicResponseState {
        users_by_id,
        users_store_path,
        db_records,
        db_records_adapter,
        db_records_store_path,
        db_records_sqlite_store_path,
        db_records_sqlite_connection,
        db_records_postgres_dsn,
        db_records_postgres_client,
        db_records_postgres_statement_cache,
        db_postgres_placeholder_max_cache,
        db_postgres_statement_cache_max,
        db_postgres_placeholder_cache_max,
        db_tx_handles,
        db_tx_max_handles,
        db_postgres_statement_timeout_ms,
        db_postgres_lock_timeout_ms,
        db_postgres_connect_timeout_ms,
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
