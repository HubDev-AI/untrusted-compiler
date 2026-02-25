use crate::lasm_db_records_log::{
    persist_lasm_dynamic_db_record_append_to_records_log,
    persist_lasm_dynamic_db_records_to_records_log,
};
use crate::lasm_db_runtime_common::{
    lasm_dynamic_postgres_client_mut, normalize_lasm_db_params,
    reconnect_lasm_dynamic_postgres_client,
};
use crate::{
    LasmDbRecord, LasmDbRecordsAdapter, LasmDynamicResponseState,
    LASM_DYNAMIC_DB_POSTGRES_RECORDS_TABLE,
};
use native_tls::TlsConnector;
use postgres::{Client as PostgresClient, NoTls};
use postgres_native_tls::MakeTlsConnector;
use rusqlite::{params, Connection};
use std::fs;
use std::path::Path;
use std::time::Duration;

pub(crate) const LASM_DB_SQLITE_BUSY_TIMEOUT_MS_DEFAULT: u64 = 2000;
pub(crate) const LASM_DB_SQLITE_JOURNAL_MODE_DEFAULT: &str = "WAL";
pub(crate) const LASM_DB_SQLITE_SYNCHRONOUS_DEFAULT: &str = "NORMAL";
pub(crate) const LASM_DB_SQLITE_JOURNAL_MODE_ENV: &str = "SEC4_RT_LASM_DB_SQLITE_JOURNAL_MODE";
pub(crate) const LASM_DB_SQLITE_SYNCHRONOUS_ENV: &str = "SEC4_RT_LASM_DB_SQLITE_SYNCHRONOUS";
pub(crate) const LASM_DB_POSTGRES_STATEMENT_TIMEOUT_MS_DEFAULT: u64 = 5000;
pub(crate) const LASM_DB_POSTGRES_LOCK_TIMEOUT_MS_DEFAULT: u64 = 2000;
pub(crate) const LASM_DB_POSTGRES_CONNECT_TIMEOUT_MS_DEFAULT: u64 = 2000;
pub(crate) const LASM_DB_POSTGRES_TLS_MODE_ENV: &str = "SEC4_RT_LASM_DB_POSTGRES_TLS_MODE";

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub(crate) enum LasmDbPostgresTlsMode {
    #[default]
    Auto,
    Disable,
    Require,
}

pub(crate) fn parse_lasm_db_postgres_tls_mode(value: &str) -> Option<LasmDbPostgresTlsMode> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Some(LasmDbPostgresTlsMode::Auto),
        "disable" | "disabled" | "off" | "none" => Some(LasmDbPostgresTlsMode::Disable),
        "require" | "required" | "on" => Some(LasmDbPostgresTlsMode::Require),
        _ => None,
    }
}

pub(crate) fn lasm_db_postgres_tls_mode_label(mode: LasmDbPostgresTlsMode) -> &'static str {
    match mode {
        LasmDbPostgresTlsMode::Auto => "auto",
        LasmDbPostgresTlsMode::Disable => "disable",
        LasmDbPostgresTlsMode::Require => "require",
    }
}

fn parse_lasm_db_sqlite_journal_mode(value: &str) -> Option<&'static str> {
    match value.trim().to_ascii_uppercase().as_str() {
        "DELETE" => Some("DELETE"),
        "TRUNCATE" => Some("TRUNCATE"),
        "PERSIST" => Some("PERSIST"),
        "MEMORY" => Some("MEMORY"),
        "WAL" => Some("WAL"),
        "OFF" => Some("OFF"),
        _ => None,
    }
}

fn parse_lasm_db_sqlite_synchronous(value: &str) -> Option<&'static str> {
    match value.trim().to_ascii_uppercase().as_str() {
        "OFF" => Some("OFF"),
        "NORMAL" => Some("NORMAL"),
        "FULL" => Some("FULL"),
        "EXTRA" => Some("EXTRA"),
        _ => None,
    }
}

pub(crate) fn normalize_lasm_db_sqlite_journal_mode(value: &str) -> Option<&'static str> {
    parse_lasm_db_sqlite_journal_mode(value)
}

pub(crate) fn normalize_lasm_db_sqlite_synchronous(value: &str) -> Option<&'static str> {
    parse_lasm_db_sqlite_synchronous(value)
}

pub(crate) fn resolve_lasm_db_sqlite_journal_mode() -> &'static str {
    let Ok(raw) = std::env::var(LASM_DB_SQLITE_JOURNAL_MODE_ENV) else {
        return LASM_DB_SQLITE_JOURNAL_MODE_DEFAULT;
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return LASM_DB_SQLITE_JOURNAL_MODE_DEFAULT;
    }
    if let Some(mode) = parse_lasm_db_sqlite_journal_mode(trimmed) {
        return mode;
    }
    eprintln!(
        "warning: invalid {} value `{}`; defaulting to {}",
        LASM_DB_SQLITE_JOURNAL_MODE_ENV,
        trimmed,
        LASM_DB_SQLITE_JOURNAL_MODE_DEFAULT
    );
    LASM_DB_SQLITE_JOURNAL_MODE_DEFAULT
}

pub(crate) fn resolve_lasm_db_sqlite_synchronous() -> &'static str {
    let Ok(raw) = std::env::var(LASM_DB_SQLITE_SYNCHRONOUS_ENV) else {
        return LASM_DB_SQLITE_SYNCHRONOUS_DEFAULT;
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return LASM_DB_SQLITE_SYNCHRONOUS_DEFAULT;
    }
    if let Some(mode) = parse_lasm_db_sqlite_synchronous(trimmed) {
        return mode;
    }
    eprintln!(
        "warning: invalid {} value `{}`; defaulting to {}",
        LASM_DB_SQLITE_SYNCHRONOUS_ENV,
        trimmed,
        LASM_DB_SQLITE_SYNCHRONOUS_DEFAULT
    );
    LASM_DB_SQLITE_SYNCHRONOUS_DEFAULT
}

fn lasm_postgres_connect_timeout_seconds_from_ms(timeout_ms: u64) -> u64 {
    timeout_ms.saturating_add(999).saturating_div(1000).max(1)
}

fn has_lasm_postgres_connect_timeout(dsn: &str) -> bool {
    let compact = dsn
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>()
        .to_ascii_lowercase();
    compact.contains("connect_timeout=")
}

fn redact_lasm_postgres_url_userinfo_password(dsn: &str) -> String {
    for scheme in ["postgres://", "postgresql://"] {
        if let Some(rest) = dsn.strip_prefix(scheme) {
            let Some(at_index) = rest.find('@') else {
                continue;
            };
            let auth = &rest[..at_index];
            let Some(colon_index) = auth.find(':') else {
                continue;
            };
            let user = &auth[..colon_index];
            let tail = &rest[at_index + 1..];
            return format!("{scheme}{user}:***@{tail}");
        }
    }
    dsn.to_string()
}

fn redact_lasm_postgres_keyword_dsn_password(dsn: &str) -> String {
    let bytes = dsn.as_bytes();
    let mut out = String::with_capacity(dsn.len());
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index].is_ascii_whitespace() {
            out.push(bytes[index] as char);
            index += 1;
            continue;
        }

        let key_start = index;
        while index < bytes.len() && !bytes[index].is_ascii_whitespace() && bytes[index] != b'=' {
            index += 1;
        }
        let key_end = index;
        let mut eq_index = index;
        while eq_index < bytes.len() && bytes[eq_index].is_ascii_whitespace() {
            eq_index += 1;
        }
        if eq_index >= bytes.len() || bytes[eq_index] != b'=' {
            out.push_str(&dsn[key_start..eq_index]);
            index = eq_index;
            continue;
        }

        let key = dsn[key_start..key_end].trim();
        out.push_str(&dsn[key_start..=eq_index]);
        index = eq_index + 1;

        let value_ws_start = index;
        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        out.push_str(&dsn[value_ws_start..index]);

        let is_password = key.eq_ignore_ascii_case("password");
        if index >= bytes.len() {
            break;
        }

        if bytes[index] == b'\'' {
            let value_start = index;
            index += 1;
            while index < bytes.len() {
                if bytes[index] == b'\'' {
                    if index + 1 < bytes.len() && bytes[index + 1] == b'\'' {
                        index += 2;
                        continue;
                    }
                    index += 1;
                    break;
                }
                index += 1;
            }
            if is_password {
                out.push_str("'***'");
            } else {
                out.push_str(&dsn[value_start..index]);
            }
            continue;
        }

        let value_start = index;
        while index < bytes.len() && !bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        if is_password {
            out.push_str("***");
        } else {
            out.push_str(&dsn[value_start..index]);
        }
    }
    out
}

fn redact_lasm_postgres_dsn_password(dsn: &str) -> String {
    let trimmed = dsn.trim();
    let url_redacted = redact_lasm_postgres_url_userinfo_password(trimmed);
    redact_lasm_postgres_keyword_dsn_password(url_redacted.as_str())
}

fn redact_lasm_postgres_connect_error_message(message: String, connect_dsn: &str) -> String {
    let redacted_dsn = redact_lasm_postgres_dsn_password(connect_dsn);
    if redacted_dsn == connect_dsn {
        return message;
    }
    message.replace(connect_dsn, redacted_dsn.as_str())
}

fn split_lasm_postgres_dsn_fragment(dsn: &str) -> (&str, Option<&str>) {
    if let Some((base, fragment)) = dsn.split_once('#') {
        (base, Some(fragment))
    } else {
        (dsn, None)
    }
}

fn build_lasm_postgres_connect_dsn(dsn: &str, connect_timeout_seconds: u64) -> String {
    let trimmed = dsn.trim();
    let (base_dsn, fragment) = split_lasm_postgres_dsn_fragment(trimmed);
    if has_lasm_postgres_connect_timeout(base_dsn) {
        return trimmed.to_string();
    }
    let rewritten_base =
        if base_dsn.starts_with("postgres://") || base_dsn.starts_with("postgresql://") {
            if base_dsn.contains('?') {
                format!("{base_dsn}&connect_timeout={connect_timeout_seconds}")
            } else {
                format!("{base_dsn}?connect_timeout={connect_timeout_seconds}")
            }
        } else {
            format!("{base_dsn} connect_timeout={connect_timeout_seconds}")
        };
    if let Some(fragment_text) = fragment {
        format!("{rewritten_base}#{fragment_text}")
    } else {
        rewritten_base
    }
}

fn is_lasm_postgres_tls_required_error(message: &str) -> bool {
    let normalized = message.to_ascii_lowercase();
    normalized.contains("ssl is required")
        || normalized.contains("requires ssl")
        || normalized.contains("sslmode=require")
        || normalized.contains("sslmode=verify-ca")
        || normalized.contains("sslmode=verify-full")
        || normalized.contains("ssl off")
}

fn build_lasm_native_tls_connector() -> Result<MakeTlsConnector, String> {
    let tls_connector = TlsConnector::builder()
        .build()
        .map_err(|tls_err| format!("native TLS connector bootstrap failed: {tls_err}"))?;
    Ok(MakeTlsConnector::new(tls_connector))
}

fn connect_lasm_dynamic_db_records_postgres_client(
    connect_dsn: &str,
    tls_mode: LasmDbPostgresTlsMode,
) -> Result<PostgresClient, String> {
    match tls_mode {
        LasmDbPostgresTlsMode::Disable => {
            PostgresClient::connect(connect_dsn, NoTls).map_err(|err| {
                let detail = redact_lasm_postgres_connect_error_message(
                    format!("{err} ({err:?})"),
                    connect_dsn,
                );
                format!("could not connect LASM dynamic postgres records store: {detail}")
            })
        }
        LasmDbPostgresTlsMode::Require => {
            let tls_connector = build_lasm_native_tls_connector().map_err(|tls_err| {
                format!("could not connect LASM dynamic postgres records store: {tls_err}")
            })?;
            PostgresClient::connect(connect_dsn, tls_connector).map_err(|err| {
                let detail = redact_lasm_postgres_connect_error_message(
                    format!("{err} ({err:?})"),
                    connect_dsn,
                );
                format!("could not connect LASM dynamic postgres records store: {detail}")
            })
        }
        LasmDbPostgresTlsMode::Auto => match PostgresClient::connect(connect_dsn, NoTls) {
            Ok(client) => Ok(client),
            Err(no_tls_err) => {
                let no_tls_message = redact_lasm_postgres_connect_error_message(
                    format!("{no_tls_err} ({no_tls_err:?})"),
                    connect_dsn,
                );
                if !is_lasm_postgres_tls_required_error(no_tls_message.as_str()) {
                    return Err(format!(
                        "could not connect LASM dynamic postgres records store: {no_tls_message}"
                    ));
                }
                let tls_connector = build_lasm_native_tls_connector().map_err(|tls_err| {
                    format!(
                        "could not connect LASM dynamic postgres records store: TLS required, but {tls_err}"
                    )
                })?;
                PostgresClient::connect(connect_dsn, tls_connector).map_err(|tls_err| {
                    let tls_detail = redact_lasm_postgres_connect_error_message(
                        format!("{tls_err} ({tls_err:?})"),
                        connect_dsn,
                    );
                    format!(
                        "could not connect LASM dynamic postgres records store: TLS retry failed after NoTLS error `{no_tls_message}`: {tls_detail}"
                    )
                })
            }
        },
    }
}

pub(crate) fn ensure_lasm_dynamic_db_records_sqlite_schema(
    connection: &Connection,
) -> rusqlite::Result<()> {
    connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS lasm_db_records (
            id INTEGER PRIMARY KEY NOT NULL,
            op TEXT NOT NULL,
            db INTEGER NOT NULL,
            template TEXT NOT NULL,
            params TEXT NOT NULL,
            tx INTEGER NOT NULL,
            affected_rows INTEGER NOT NULL DEFAULT 0,
            created_at_ms INTEGER NOT NULL
        );",
    )?;
    let mut statement = connection.prepare("PRAGMA table_info(lasm_db_records)")?;
    let mut rows = statement.query([])?;
    let mut has_affected_rows = false;
    while let Some(row) = rows.next()? {
        let name: String = row.get(1)?;
        if name == "affected_rows" {
            has_affected_rows = true;
            break;
        }
    }
    drop(rows);
    drop(statement);
    if !has_affected_rows {
        connection.execute_batch(
            "ALTER TABLE lasm_db_records ADD COLUMN affected_rows INTEGER NOT NULL DEFAULT 0;",
        )?;
    }
    Ok(())
}

pub(crate) fn load_lasm_dynamic_db_records_from_sqlite(path: &Path) -> Vec<LasmDbRecord> {
    if !path.exists() {
        return Vec::new();
    }
    let connection = match Connection::open(path) {
        Ok(connection) => connection,
        Err(err) => {
            eprintln!(
                "warning: LASM dynamic sqlite records store open failed at `{}`: {err}",
                path.display()
            );
            return Vec::new();
        }
    };
    if let Err(err) = ensure_lasm_dynamic_db_records_sqlite_schema(&connection) {
        eprintln!(
            "warning: LASM dynamic sqlite records schema check failed at `{}`: {err}",
            path.display()
        );
        return Vec::new();
    }
    let mut statement = match connection.prepare(
        "SELECT id, op, db, template, params, tx, affected_rows, created_at_ms \
         FROM lasm_db_records \
         ORDER BY id ASC",
    ) {
        Ok(statement) => statement,
        Err(err) => {
            eprintln!(
                "warning: LASM dynamic sqlite records query prep failed at `{}`: {err}",
                path.display()
            );
            return Vec::new();
        }
    };
    let entries = match statement.query_map([], |row| {
        let id = row.get::<_, i64>(0)?;
        if id < 0 {
            return Err(rusqlite::Error::IntegralValueOutOfRange(0, id));
        }
        let affected_rows = row.get::<_, i64>(6)?;
        if affected_rows < 0 {
            return Err(rusqlite::Error::IntegralValueOutOfRange(6, affected_rows));
        }
        let created_at_ms = row.get::<_, i64>(7)?;
        if created_at_ms < 0 {
            return Err(rusqlite::Error::IntegralValueOutOfRange(7, created_at_ms));
        }
        let params: String = row.get(4)?;
        Ok(LasmDbRecord {
            id: id as u64,
            op: row.get(1)?,
            db: row.get(2)?,
            template: row.get(3)?,
            params: normalize_lasm_db_record_loaded_params(params),
            tx: row.get(5)?,
            affected_rows: affected_rows as u64,
            created_at_ms: created_at_ms as u64,
        })
    }) {
        Ok(entries) => entries,
        Err(err) => {
            eprintln!(
                "warning: LASM dynamic sqlite records query failed at `{}`: {err}",
                path.display()
            );
            return Vec::new();
        }
    };
    let mut records = Vec::new();
    for (index, entry) in entries.enumerate() {
        match entry {
            Ok(record) => records.push(record),
            Err(err) => {
                eprintln!(
                    "warning: LASM dynamic sqlite records parse failed at `{}` row {}: {err}",
                    path.display(),
                    index + 1
                );
            }
        }
    }
    records
}

pub(crate) fn connect_lasm_dynamic_db_records_sqlite(
    path: &Path,
    busy_timeout_ms: u64,
    sqlite_journal_mode: &str,
    sqlite_synchronous: &str,
) -> Result<Connection, String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            format!(
                "could not create LASM dynamic sqlite records store directory `{}`: {err}",
                parent.display()
            )
        })?;
    }
    let connection = Connection::open(path).map_err(|err| {
        format!(
            "could not open LASM dynamic sqlite records store `{}`: {err}",
            path.display()
        )
    })?;
    let busy_timeout_ms = busy_timeout_ms.max(1);
    connection
        .busy_timeout(Duration::from_millis(busy_timeout_ms))
        .map_err(|err| {
            format!(
                "could not set LASM dynamic sqlite busy timeout `{}`: {err}",
                path.display()
            )
        })?;
    connection
        .execute_batch(
            format!(
                "PRAGMA foreign_keys = ON; PRAGMA journal_mode = {}; PRAGMA synchronous = {};",
                sqlite_journal_mode,
                sqlite_synchronous,
            )
            .as_str(),
        )
        .map_err(|err| {
            format!(
                "could not configure LASM dynamic sqlite pragmas `{}`: {err}",
                path.display()
            )
        })?;
    ensure_lasm_dynamic_db_records_sqlite_schema(&connection).map_err(|err| {
        format!(
            "could not initialize LASM dynamic sqlite records schema `{}`: {err}",
            path.display()
        )
    })?;
    Ok(connection)
}

pub(crate) fn connect_lasm_dynamic_db_records_postgres(
    dsn: &str,
    tls_mode: LasmDbPostgresTlsMode,
    statement_timeout_ms: u64,
    lock_timeout_ms: u64,
    connect_timeout_ms: u64,
) -> Result<PostgresClient, String> {
    let connect_timeout_seconds =
        lasm_postgres_connect_timeout_seconds_from_ms(connect_timeout_ms.max(1));
    let connect_dsn = build_lasm_postgres_connect_dsn(dsn, connect_timeout_seconds);
    let mut client =
        connect_lasm_dynamic_db_records_postgres_client(connect_dsn.as_str(), tls_mode)?;
    let statement_timeout_ms = statement_timeout_ms.max(1);
    let lock_timeout_ms = lock_timeout_ms.max(1);
    let timeout_settings = format!(
        "SET statement_timeout = {statement_timeout_ms}; SET lock_timeout = {lock_timeout_ms};"
    );
    client
        .batch_execute(timeout_settings.as_str())
        .map_err(|err| {
            format!("could not configure LASM dynamic postgres session timeouts: {err}")
        })?;
    Ok(client)
}

#[cfg(test)]
mod tests {
    use super::{
        build_lasm_postgres_connect_dsn, is_lasm_postgres_tls_required_error,
        has_lasm_postgres_connect_timeout, lasm_db_postgres_tls_mode_label,
        lasm_postgres_connect_timeout_seconds_from_ms,
        normalize_lasm_db_record_loaded_params, parse_lasm_db_postgres_tls_mode,
        parse_lasm_db_sqlite_journal_mode, parse_lasm_db_sqlite_synchronous,
        redact_lasm_postgres_connect_error_message, redact_lasm_postgres_dsn_password,
        split_lasm_postgres_dsn_fragment, LasmDbPostgresTlsMode,
    };

    #[test]
    fn postgres_connect_timeout_seconds_rounds_up_from_millis() {
        assert_eq!(lasm_postgres_connect_timeout_seconds_from_ms(1), 1);
        assert_eq!(lasm_postgres_connect_timeout_seconds_from_ms(1000), 1);
        assert_eq!(lasm_postgres_connect_timeout_seconds_from_ms(1001), 2);
    }

    #[test]
    fn postgres_connect_dsn_injects_timeout_for_url_without_query() {
        let rewritten = build_lasm_postgres_connect_dsn("postgres://u:p@localhost/db", 2);
        assert_eq!(rewritten, "postgres://u:p@localhost/db?connect_timeout=2");
    }

    #[test]
    fn postgres_connect_dsn_injects_timeout_for_url_with_query() {
        let rewritten =
            build_lasm_postgres_connect_dsn("postgres://u:p@localhost/db?sslmode=disable", 3);
        assert_eq!(
            rewritten,
            "postgres://u:p@localhost/db?sslmode=disable&connect_timeout=3"
        );
    }

    #[test]
    fn postgres_connect_dsn_injects_timeout_for_keyword_dsn() {
        let rewritten = build_lasm_postgres_connect_dsn("host=localhost dbname=sec4", 5);
        assert_eq!(rewritten, "host=localhost dbname=sec4 connect_timeout=5");
    }

    #[test]
    fn postgres_connect_dsn_preserves_existing_connect_timeout() {
        let rewritten = build_lasm_postgres_connect_dsn(
            "postgres://u:p@localhost/db?connect_timeout=9&sslmode=disable",
            2,
        );
        assert_eq!(
            rewritten,
            "postgres://u:p@localhost/db?connect_timeout=9&sslmode=disable"
        );
    }

    #[test]
    fn postgres_connect_dsn_preserves_existing_connect_timeout_case_insensitive() {
        let rewritten = build_lasm_postgres_connect_dsn(
            "postgres://u:p@localhost/db?CONNECT_TIMEOUT=9&sslmode=disable",
            2,
        );
        assert_eq!(
            rewritten,
            "postgres://u:p@localhost/db?CONNECT_TIMEOUT=9&sslmode=disable"
        );
    }

    #[test]
    fn postgres_connect_dsn_preserves_existing_connect_timeout_with_keyword_spacing() {
        let rewritten = build_lasm_postgres_connect_dsn(
            "host=localhost dbname=sec4 connect_timeout = 9",
            2,
        );
        assert_eq!(rewritten, "host=localhost dbname=sec4 connect_timeout = 9");
    }

    #[test]
    fn postgres_connect_timeout_detector_handles_whitespace_and_case() {
        assert!(has_lasm_postgres_connect_timeout(
            "postgres://u:p@localhost/db?CONNECT_TIMEOUT=9&sslmode=disable"
        ));
        assert!(has_lasm_postgres_connect_timeout(
            "host=localhost dbname=sec4 connect_timeout = 9"
        ));
        assert!(!has_lasm_postgres_connect_timeout(
            "postgres://u:p@localhost/db?sslmode=disable"
        ));
    }

    #[test]
    fn postgres_connect_dsn_injects_timeout_before_fragment_without_query() {
        let rewritten = build_lasm_postgres_connect_dsn("postgres://u:p@localhost/db#frag", 2);
        assert_eq!(
            rewritten,
            "postgres://u:p@localhost/db?connect_timeout=2#frag"
        );
    }

    #[test]
    fn postgres_connect_dsn_injects_timeout_before_fragment_with_query() {
        let rewritten =
            build_lasm_postgres_connect_dsn("postgres://u:p@localhost/db?sslmode=disable#frag", 3);
        assert_eq!(
            rewritten,
            "postgres://u:p@localhost/db?sslmode=disable&connect_timeout=3#frag"
        );
    }

    #[test]
    fn postgres_connect_dsn_preserves_existing_connect_timeout_with_fragment() {
        let rewritten = build_lasm_postgres_connect_dsn(
            "postgres://u:p@localhost/db?CONNECT_TIMEOUT=9&sslmode=disable#frag",
            2,
        );
        assert_eq!(
            rewritten,
            "postgres://u:p@localhost/db?CONNECT_TIMEOUT=9&sslmode=disable#frag"
        );
    }

    #[test]
    fn split_postgres_dsn_fragment_handles_fragment_and_non_fragment_cases() {
        assert_eq!(
            split_lasm_postgres_dsn_fragment("postgres://u:p@localhost/db#frag"),
            ("postgres://u:p@localhost/db", Some("frag"))
        );
        assert_eq!(
            split_lasm_postgres_dsn_fragment("postgres://u:p@localhost/db"),
            ("postgres://u:p@localhost/db", None)
        );
    }

    #[test]
    fn redact_postgres_dsn_password_masks_url_credentials() {
        assert_eq!(
            redact_lasm_postgres_dsn_password(
                "postgres://sec4:sec4dev@127.0.0.1:5432/sec4_local?sslmode=disable"
            ),
            "postgres://sec4:***@127.0.0.1:5432/sec4_local?sslmode=disable"
        );
        assert_eq!(
            redact_lasm_postgres_dsn_password("host=localhost dbname=sec4"),
            "host=localhost dbname=sec4"
        );
    }

    #[test]
    fn redact_postgres_dsn_password_masks_keyword_conninfo_password() {
        assert_eq!(
            redact_lasm_postgres_dsn_password(
                "host=localhost user=sec4 password=sec4dev dbname=sec4_local"
            ),
            "host=localhost user=sec4 password=*** dbname=sec4_local"
        );
        assert_eq!(
            redact_lasm_postgres_dsn_password(
                "host=localhost user=sec4 PASSWORD = sec4dev dbname=sec4_local"
            ),
            "host=localhost user=sec4 PASSWORD = *** dbname=sec4_local"
        );
    }

    #[test]
    fn redact_postgres_dsn_password_masks_quoted_keyword_password() {
        assert_eq!(
            redact_lasm_postgres_dsn_password(
                "host=localhost user=sec4 password='sec4 dev secret' dbname=sec4_local"
            ),
            "host=localhost user=sec4 password='***' dbname=sec4_local"
        );
    }

    #[test]
    fn redact_postgres_connect_error_message_replaces_embedded_dsn() {
        let dsn = "postgres://sec4:sec4dev@127.0.0.1:5432/sec4_local?sslmode=disable";
        let raw_message = format!("connection refused for dsn {dsn}");
        let redacted = redact_lasm_postgres_connect_error_message(raw_message, dsn);
        assert_eq!(
            redacted,
            "connection refused for dsn postgres://sec4:***@127.0.0.1:5432/sec4_local?sslmode=disable"
        );
    }

    #[test]
    fn redact_postgres_connect_error_message_replaces_keyword_conninfo_dsn() {
        let dsn = "host=localhost user=sec4 password=sec4dev dbname=sec4_local";
        let raw_message = format!("connection refused for dsn {dsn}");
        let redacted = redact_lasm_postgres_connect_error_message(raw_message, dsn);
        assert_eq!(
            redacted,
            "connection refused for dsn host=localhost user=sec4 password=*** dbname=sec4_local"
        );
    }

    #[test]
    fn postgres_tls_error_detector_matches_required_ssl_patterns() {
        assert!(is_lasm_postgres_tls_required_error(
            "no pg_hba.conf entry for host \"127.0.0.1\", user \"u\", database \"db\", SSL off"
        ));
        assert!(is_lasm_postgres_tls_required_error(
            "error: ssl is required by server"
        ));
        assert!(is_lasm_postgres_tls_required_error(
            "invalid configuration: sslmode=require"
        ));
    }

    #[test]
    fn postgres_tls_error_detector_ignores_non_ssl_failures() {
        assert!(!is_lasm_postgres_tls_required_error(
            "password authentication failed for user"
        ));
        assert!(!is_lasm_postgres_tls_required_error(
            "database does not exist"
        ));
    }

    #[test]
    fn parse_postgres_tls_mode_supports_operator_aliases() {
        assert_eq!(
            parse_lasm_db_postgres_tls_mode("auto"),
            Some(LasmDbPostgresTlsMode::Auto)
        );
        assert_eq!(
            parse_lasm_db_postgres_tls_mode("none"),
            Some(LasmDbPostgresTlsMode::Disable)
        );
        assert_eq!(
            parse_lasm_db_postgres_tls_mode("required"),
            Some(LasmDbPostgresTlsMode::Require)
        );
        assert_eq!(parse_lasm_db_postgres_tls_mode("invalid"), None);
    }

    #[test]
    fn parse_sqlite_journal_mode_supports_known_values() {
        assert_eq!(parse_lasm_db_sqlite_journal_mode("wal"), Some("WAL"));
        assert_eq!(parse_lasm_db_sqlite_journal_mode("delete"), Some("DELETE"));
        assert_eq!(parse_lasm_db_sqlite_journal_mode("truncate"), Some("TRUNCATE"));
        assert_eq!(parse_lasm_db_sqlite_journal_mode("persist"), Some("PERSIST"));
        assert_eq!(parse_lasm_db_sqlite_journal_mode("memory"), Some("MEMORY"));
        assert_eq!(parse_lasm_db_sqlite_journal_mode("off"), Some("OFF"));
        assert_eq!(parse_lasm_db_sqlite_journal_mode("bogus"), None);
    }

    #[test]
    fn parse_sqlite_synchronous_supports_known_values() {
        assert_eq!(parse_lasm_db_sqlite_synchronous("off"), Some("OFF"));
        assert_eq!(parse_lasm_db_sqlite_synchronous("normal"), Some("NORMAL"));
        assert_eq!(parse_lasm_db_sqlite_synchronous("full"), Some("FULL"));
        assert_eq!(parse_lasm_db_sqlite_synchronous("extra"), Some("EXTRA"));
        assert_eq!(parse_lasm_db_sqlite_synchronous("bogus"), None);
    }

    #[test]
    fn postgres_tls_mode_labels_are_stable() {
        assert_eq!(
            lasm_db_postgres_tls_mode_label(LasmDbPostgresTlsMode::Auto),
            "auto"
        );
        assert_eq!(
            lasm_db_postgres_tls_mode_label(LasmDbPostgresTlsMode::Disable),
            "disable"
        );
        assert_eq!(
            lasm_db_postgres_tls_mode_label(LasmDbPostgresTlsMode::Require),
            "require"
        );
    }

    #[test]
    fn normalize_loaded_params_canonicalizes_json() {
        let normalized = normalize_lasm_db_record_loaded_params("{\"b\":2,\"a\":1}".to_string());
        assert_eq!(normalized, "{\"a\":1,\"b\":2}");
    }

    #[test]
    fn normalize_loaded_params_preserves_non_json_text() {
        let normalized = normalize_lasm_db_record_loaded_params("alpha".to_string());
        assert_eq!(normalized, "alpha");
    }
}

pub(crate) fn ensure_lasm_dynamic_db_records_postgres_schema(
    client: &mut PostgresClient,
) -> Result<(), String> {
    let statement = format!(
        "CREATE TABLE IF NOT EXISTS {} (
            id BIGINT PRIMARY KEY NOT NULL,
            op TEXT NOT NULL,
            db BIGINT NOT NULL,
            template TEXT NOT NULL,
            params TEXT NOT NULL,
            tx BIGINT NOT NULL,
            affected_rows BIGINT NOT NULL DEFAULT 0,
            created_at_ms BIGINT NOT NULL
        );
        ALTER TABLE {} ADD COLUMN IF NOT EXISTS affected_rows BIGINT NOT NULL DEFAULT 0;",
        LASM_DYNAMIC_DB_POSTGRES_RECORDS_TABLE, LASM_DYNAMIC_DB_POSTGRES_RECORDS_TABLE
    );
    client
        .batch_execute(statement.as_str())
        .map_err(|err| format!("could not initialize LASM dynamic postgres records schema: {err}"))
}

pub(crate) fn load_lasm_dynamic_db_records_from_postgres(
    client: &mut PostgresClient,
) -> Result<Vec<LasmDbRecord>, String> {
    let query = format!(
        "SELECT id, op, db, template, params, tx, affected_rows, created_at_ms \
         FROM {} \
         ORDER BY id ASC",
        LASM_DYNAMIC_DB_POSTGRES_RECORDS_TABLE
    );
    let rows = client
        .query(query.as_str(), &[])
        .map_err(|err| format!("could not query LASM dynamic postgres records store: {err}"))?;
    let mut records = Vec::with_capacity(rows.len());
    for (index, row) in rows.into_iter().enumerate() {
        let id: i64 = row.get(0);
        let affected_rows: i64 = row.get(6);
        let created_at_ms: i64 = row.get(7);
        if id < 0 {
            return Err(format!(
                "could not load LASM dynamic postgres record row {}: id out of u64 range",
                index + 1
            ));
        }
        if affected_rows < 0 {
            return Err(format!(
                "could not load LASM dynamic postgres record row {}: affected_rows out of u64 range",
                index + 1
            ));
        }
        if created_at_ms < 0 {
            return Err(format!(
                "could not load LASM dynamic postgres record row {}: created_at_ms out of u64 range",
                index + 1
            ));
        }
        let params: String = row.get(4);
        records.push(LasmDbRecord {
            id: id as u64,
            op: row.get(1),
            db: row.get(2),
            template: row.get(3),
            params: normalize_lasm_db_record_loaded_params(params),
            tx: row.get(5),
            affected_rows: affected_rows as u64,
            created_at_ms: created_at_ms as u64,
        });
    }
    Ok(records)
}

fn normalize_lasm_db_record_loaded_params(params: String) -> String {
    normalize_lasm_db_params(params.as_str())
}

pub(crate) fn persist_lasm_dynamic_db_records_to_sqlite(
    state: &mut LasmDynamicResponseState,
) -> Result<(), String> {
    let Some(path) = state.db_records_sqlite_store_path.as_ref() else {
        return Ok(());
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            format!(
                "could not create LASM dynamic sqlite records store directory `{}`: {err}",
                parent.display()
            )
        })?;
    }
    let mut connection = Connection::open(path).map_err(|err| {
        format!(
            "could not open LASM dynamic sqlite records store `{}`: {err}",
            path.display()
        )
    })?;
    ensure_lasm_dynamic_db_records_sqlite_schema(&connection).map_err(|err| {
        format!(
            "could not initialize LASM dynamic sqlite records schema `{}`: {err}",
            path.display()
        )
    })?;
    let tx = connection.transaction().map_err(|err| {
        format!(
            "could not start LASM dynamic sqlite records transaction `{}`: {err}",
            path.display()
        )
    })?;
    tx.execute("DELETE FROM lasm_db_records", [])
        .map_err(|err| {
            format!(
                "could not clear LASM dynamic sqlite records store `{}`: {err}",
                path.display()
            )
        })?;
    let mut ordered = state.db_records.clone();
    ordered.sort_by_key(|record| record.id);
    let mut statement = tx
        .prepare(
            "INSERT INTO lasm_db_records \
             (id, op, db, template, params, tx, affected_rows, created_at_ms) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        )
        .map_err(|err| {
            format!(
                "could not prepare LASM dynamic sqlite records insert `{}`: {err}",
                path.display()
            )
        })?;
    for record in &ordered {
        let id = i64::try_from(record.id).map_err(|_| {
            format!(
                "could not persist LASM dynamic sqlite record id {}: out of i64 range",
                record.id
            )
        })?;
        let created_at_ms = i64::try_from(record.created_at_ms).map_err(|_| {
            format!(
                "could not persist LASM dynamic sqlite record timestamp {}: out of i64 range",
                record.created_at_ms
            )
        })?;
        let affected_rows = i64::try_from(record.affected_rows).map_err(|_| {
            format!(
                "could not persist LASM dynamic sqlite record affected_rows {}: out of i64 range",
                record.affected_rows
            )
        })?;
        statement
            .execute(params![
                id,
                record.op.as_str(),
                record.db,
                record.template.as_str(),
                record.params.as_str(),
                record.tx,
                affected_rows,
                created_at_ms
            ])
            .map_err(|err| {
                format!(
                    "could not insert LASM dynamic sqlite record {} into `{}`: {err}",
                    record.id,
                    path.display()
                )
            })?;
    }
    drop(statement);
    tx.commit().map_err(|err| {
        format!(
            "could not commit LASM dynamic sqlite records store `{}`: {err}",
            path.display()
        )
    })?;
    Ok(())
}

pub(crate) fn persist_lasm_dynamic_db_records_to_postgres(
    state: &mut LasmDynamicResponseState,
) -> Result<(), String> {
    let mut ordered = state.db_records.clone();
    ordered.sort_by_key(|record| record.id);

    let run_sync = |client: &mut PostgresClient| -> Result<(), String> {
        ensure_lasm_dynamic_db_records_postgres_schema(client)?;
        let mut tx = client.transaction().map_err(|err| {
            format!("could not start LASM dynamic postgres records transaction: {err}")
        })?;
        let delete_statement = format!("DELETE FROM {}", LASM_DYNAMIC_DB_POSTGRES_RECORDS_TABLE);
        tx.execute(delete_statement.as_str(), &[])
            .map_err(|err| format!("could not clear LASM dynamic postgres records store: {err}"))?;
        let insert_statement = format!(
            "INSERT INTO {} (id, op, db, template, params, tx, affected_rows, created_at_ms) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            LASM_DYNAMIC_DB_POSTGRES_RECORDS_TABLE
        );
        for record in &ordered {
            let id = i64::try_from(record.id).map_err(|_| {
                format!(
                    "could not persist LASM dynamic postgres record id {}: out of i64 range",
                    record.id
                )
            })?;
            let created_at_ms = i64::try_from(record.created_at_ms).map_err(|_| {
                format!(
                    "could not persist LASM dynamic postgres record timestamp {}: out of i64 range",
                    record.created_at_ms
                )
            })?;
            let affected_rows = i64::try_from(record.affected_rows).map_err(|_| {
                format!(
                    "could not persist LASM dynamic postgres record affected_rows {}: out of i64 range",
                    record.affected_rows
                )
            })?;
            tx.execute(
                insert_statement.as_str(),
                &[
                    &id,
                    &record.op,
                    &record.db,
                    &record.template,
                    &record.params,
                    &record.tx,
                    &affected_rows,
                    &created_at_ms,
                ],
            )
            .map_err(|err| {
                format!(
                    "could not insert LASM dynamic postgres record {}: {err}",
                    record.id
                )
            })?;
        }
        tx.commit().map_err(|err| {
            format!("could not commit LASM dynamic postgres records store: {err}")
        })?;
        Ok(())
    };

    let initial = {
        let client = lasm_dynamic_postgres_client_mut(state)?;
        run_sync(client)
    };
    match initial {
        Ok(()) => {}
        Err(message) if message.contains("closed") || message.contains("broken pipe") => {
            reconnect_lasm_dynamic_postgres_client(state)?;
            let client = lasm_dynamic_postgres_client_mut(state)?;
            run_sync(client)?;
        }
        Err(message) => return Err(message),
    }
    Ok(())
}

pub(crate) fn persist_lasm_dynamic_db_record_append_to_sqlite(
    state: &mut LasmDynamicResponseState,
    record: &LasmDbRecord,
) -> Result<(), String> {
    let Some(path) = state.db_records_sqlite_store_path.as_ref().cloned() else {
        return Ok(());
    };
    if state.db_records_sqlite_connection.is_none() {
        let connection = connect_lasm_dynamic_db_records_sqlite(
            path.as_path(),
            state.db_sqlite_busy_timeout_ms.max(1),
            state.db_sqlite_journal_mode.as_str(),
            state.db_sqlite_synchronous.as_str(),
        )?;
        state.db_records_sqlite_connection = Some(connection);
    }
    let id = i64::try_from(record.id).map_err(|_| {
        format!(
            "could not persist LASM dynamic sqlite record id {}: out of i64 range",
            record.id
        )
    })?;
    let created_at_ms = i64::try_from(record.created_at_ms).map_err(|_| {
        format!(
            "could not persist LASM dynamic sqlite record timestamp {}: out of i64 range",
            record.created_at_ms
        )
    })?;
    let affected_rows = i64::try_from(record.affected_rows).map_err(|_| {
        format!(
            "could not persist LASM dynamic sqlite record affected_rows {}: out of i64 range",
            record.affected_rows
        )
    })?;
    let result = match state.db_records_sqlite_connection.as_mut() {
        Some(connection) => connection.execute(
            "INSERT INTO lasm_db_records \
             (id, op, db, template, params, tx, affected_rows, created_at_ms) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8) \
             ON CONFLICT(id) DO UPDATE SET \
                 op = excluded.op, \
                 db = excluded.db, \
                 template = excluded.template, \
                 params = excluded.params, \
                 tx = excluded.tx, \
                 affected_rows = excluded.affected_rows, \
                 created_at_ms = excluded.created_at_ms",
            params![
                id,
                record.op.as_str(),
                record.db,
                record.template.as_str(),
                record.params.as_str(),
                record.tx,
                affected_rows,
                created_at_ms
            ],
        ),
        None => Err(rusqlite::Error::InvalidQuery),
    };
    match result {
        Ok(_) => Ok(()),
        Err(err) => {
            state.db_records_sqlite_connection = None;
            let append_error = format!(
                "could not append LASM dynamic sqlite record {} into `{}`: {err}",
                record.id,
                path.display()
            );
            persist_lasm_dynamic_db_records_to_sqlite(state).map_err(|full_sync_error| {
                format!("{append_error}; full sqlite sync fallback failed: {full_sync_error}")
            })
        }
    }
}

pub(crate) fn persist_lasm_dynamic_db_record_append_to_postgres(
    state: &mut LasmDynamicResponseState,
    record: &LasmDbRecord,
) -> Result<(), String> {
    let id = i64::try_from(record.id).map_err(|_| {
        format!(
            "could not persist LASM dynamic postgres record id {}: out of i64 range",
            record.id
        )
    })?;
    let created_at_ms = i64::try_from(record.created_at_ms).map_err(|_| {
        format!(
            "could not persist LASM dynamic postgres record timestamp {}: out of i64 range",
            record.created_at_ms
        )
    })?;
    let affected_rows = i64::try_from(record.affected_rows).map_err(|_| {
        format!(
            "could not persist LASM dynamic postgres record affected_rows {}: out of i64 range",
            record.affected_rows
        )
    })?;
    let run_sync = |client: &mut PostgresClient| -> Result<(), String> {
        ensure_lasm_dynamic_db_records_postgres_schema(client)?;
        let insert_statement = format!(
            "INSERT INTO {} (id, op, db, template, params, tx, affected_rows, created_at_ms) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
             ON CONFLICT (id) DO UPDATE SET \
                 op = EXCLUDED.op, \
                 db = EXCLUDED.db, \
                 template = EXCLUDED.template, \
                 params = EXCLUDED.params, \
                 tx = EXCLUDED.tx, \
                 affected_rows = EXCLUDED.affected_rows, \
                 created_at_ms = EXCLUDED.created_at_ms",
            LASM_DYNAMIC_DB_POSTGRES_RECORDS_TABLE
        );
        client
            .execute(
                insert_statement.as_str(),
                &[
                    &id,
                    &record.op,
                    &record.db,
                    &record.template,
                    &record.params,
                    &record.tx,
                    &affected_rows,
                    &created_at_ms,
                ],
            )
            .map_err(|err| {
                format!(
                    "could not append LASM dynamic postgres record {}: {err}",
                    record.id
                )
            })?;
        Ok(())
    };
    let initial = {
        let client = lasm_dynamic_postgres_client_mut(state)?;
        run_sync(client)
    };
    match initial {
        Ok(()) => Ok(()),
        Err(message) if message.contains("closed") || message.contains("broken pipe") => {
            reconnect_lasm_dynamic_postgres_client(state)?;
            let client = lasm_dynamic_postgres_client_mut(state)?;
            run_sync(client).or_else(|retry_error| {
                persist_lasm_dynamic_db_records_to_postgres(state).map_err(|full_sync_error| {
                    format!(
                        "{retry_error}; full postgres sync fallback failed after reconnect: \
                         {full_sync_error}"
                    )
                })
            })
        }
        Err(message) => {
            persist_lasm_dynamic_db_records_to_postgres(state).map_err(|full_sync_error| {
                format!("{message}; full postgres sync fallback failed: {full_sync_error}")
            })
        }
    }
}

pub(crate) fn persist_lasm_dynamic_db_record_append(
    state: &mut LasmDynamicResponseState,
    record: &LasmDbRecord,
) -> Result<(), String> {
    match state.db_records_adapter {
        LasmDbRecordsAdapter::RecordsLog => persist_lasm_dynamic_db_record_append_to_records_log(
            state.db_records_store_path.as_deref(),
            record,
        )
        .or_else(|append_error| {
            persist_lasm_dynamic_db_records_to_records_log(
                state.db_records_store_path.as_deref(),
                &state.db_records,
            )
            .map_err(|full_sync_error| {
                format!("{append_error}; full records.log sync fallback failed: {full_sync_error}")
            })
        }),
        LasmDbRecordsAdapter::Sqlite => {
            persist_lasm_dynamic_db_record_append_to_sqlite(state, record)
        }
        LasmDbRecordsAdapter::Postgres => {
            persist_lasm_dynamic_db_record_append_to_postgres(state, record)
        }
    }
}

pub(crate) fn persist_lasm_dynamic_db_records_full_sync(
    state: &mut LasmDynamicResponseState,
) -> Result<(), String> {
    match state.db_records_adapter {
        LasmDbRecordsAdapter::RecordsLog => persist_lasm_dynamic_db_records_to_records_log(
            state.db_records_store_path.as_deref(),
            &state.db_records,
        ),
        LasmDbRecordsAdapter::Sqlite => persist_lasm_dynamic_db_records_to_sqlite(state),
        LasmDbRecordsAdapter::Postgres => persist_lasm_dynamic_db_records_to_postgres(state),
    }
}
