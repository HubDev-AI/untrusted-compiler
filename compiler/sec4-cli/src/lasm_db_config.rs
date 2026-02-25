use crate::LasmDbRecordsAdapter;
use std::fs;
use std::path::{Path, PathBuf};

const LASM_DB_TX_MAX_HANDLES_DEFAULT: usize = 256;

pub(crate) fn resolve_lasm_dynamic_store_base(explicit_db_base: Option<&Path>) -> Option<PathBuf> {
    if let Some(base) = explicit_db_base {
        return Some(base.to_path_buf());
    }
    let raw = std::env::var("SEC4_RT_LASM_DB_BASE").ok()?;
    let value = raw.trim();
    if value.is_empty() {
        return None;
    }
    Some(PathBuf::from(value))
}

pub(crate) fn resolve_lasm_dynamic_db_records_adapter(
    explicit_db_records_adapter: Option<LasmDbRecordsAdapter>,
) -> LasmDbRecordsAdapter {
    if let Some(adapter) = explicit_db_records_adapter {
        return adapter;
    }
    let Ok(raw) = std::env::var("SEC4_RT_LASM_DB_ADAPTER") else {
        return LasmDbRecordsAdapter::RecordsLog;
    };
    let normalized = raw.trim().to_ascii_lowercase();
    if normalized.is_empty()
        || normalized == "records"
        || normalized == "records.log"
        || normalized == "records-log"
    {
        return LasmDbRecordsAdapter::RecordsLog;
    }
    if normalized == "sqlite" {
        return LasmDbRecordsAdapter::Sqlite;
    }
    if normalized == "postgres" || normalized == "pg" {
        return LasmDbRecordsAdapter::Postgres;
    }
    eprintln!(
        "warning: unsupported SEC4_RT_LASM_DB_ADAPTER value `{}`; defaulting to records.log adapter",
        raw.trim()
    );
    LasmDbRecordsAdapter::RecordsLog
}

pub(crate) fn resolve_lasm_dynamic_db_tx_max_handles(
    explicit_db_tx_max_handles: Option<usize>,
) -> Result<usize, String> {
    if let Some(value) = explicit_db_tx_max_handles {
        if value == 0 {
            return Err("invalid --db-max-tx-handles: expected usize >= 1".to_string());
        }
        return Ok(value);
    }
    let Ok(raw) = std::env::var("SEC4_RT_LASM_DB_MAX_TX_HANDLES") else {
        return Ok(LASM_DB_TX_MAX_HANDLES_DEFAULT);
    };
    let value = raw.trim();
    if value.is_empty() {
        return Ok(LASM_DB_TX_MAX_HANDLES_DEFAULT);
    }
    let parsed = value
        .parse::<usize>()
        .map_err(|_| "invalid SEC4_RT_LASM_DB_MAX_TX_HANDLES: expected usize >= 1".to_string())?;
    if parsed == 0 {
        return Err("invalid SEC4_RT_LASM_DB_MAX_TX_HANDLES: expected usize >= 1".to_string());
    }
    Ok(parsed)
}

fn resolve_lasm_db_postgres_dsn_from_file_contents(
    source: &str,
    raw_contents: &str,
) -> Result<String, String> {
    let mut dsn: Option<String> = None;
    for line in raw_contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if dsn.is_some() {
            return Err(format!(
                "{source} must contain exactly one DSN line (excluding comments/blank lines)"
            ));
        }
        dsn = Some(trimmed.to_string());
    }
    dsn.ok_or_else(|| format!("{source} must contain a non-empty DSN"))
}

pub(crate) fn load_lasm_db_postgres_dsn_from_file(path: &Path) -> Result<String, String> {
    let raw = fs::read_to_string(path).map_err(|err| {
        format!(
            "could not read --db-postgres-dsn-file `{}`: {err}",
            path.display()
        )
    })?;
    resolve_lasm_db_postgres_dsn_from_file_contents(
        format!("--db-postgres-dsn-file `{}`", path.display()).as_str(),
        raw.as_str(),
    )
}

pub(crate) fn resolve_lasm_dynamic_db_postgres_dsn(
    adapter: LasmDbRecordsAdapter,
    explicit_dsn: Option<&str>,
    project_path: Option<&Path>,
) -> Result<Option<String>, String> {
    if adapter != LasmDbRecordsAdapter::Postgres {
        return Ok(None);
    }
    if let Some(explicit_dsn) = explicit_dsn {
        let dsn = explicit_dsn.trim();
        if dsn.is_empty() {
            return Err(
                "db adapter postgres requires SEC4_RT_LASM_DB_POSTGRES_DSN to be set".to_string(),
            );
        }
        return Ok(Some(dsn.to_string()));
    }
    if let Ok(raw) = std::env::var("SEC4_RT_LASM_DB_POSTGRES_DSN") {
        let dsn = raw.trim().to_string();
        if dsn.is_empty() {
            return Err(
                "db adapter postgres requires SEC4_RT_LASM_DB_POSTGRES_DSN to be set".to_string(),
            );
        }
        return Ok(Some(dsn));
    }
    if let Ok(raw_file_path) = std::env::var("SEC4_RT_LASM_DB_POSTGRES_DSN_FILE") {
        let file_path_value = raw_file_path.trim();
        if file_path_value.is_empty() {
            return Err("SEC4_RT_LASM_DB_POSTGRES_DSN_FILE must not be empty".to_string());
        }
        let mut file_path = PathBuf::from(file_path_value);
        if file_path.is_relative() && !file_path.exists() {
            if let Some(project_path) = project_path {
                let candidate = project_path.join(file_path.as_path());
                if candidate.exists() {
                    file_path = candidate;
                }
            }
        }
        let dsn = fs::read_to_string(&file_path).map_err(|err| {
            format!(
                "could not read SEC4_RT_LASM_DB_POSTGRES_DSN_FILE `{}`: {err}",
                file_path.display()
            )
        })?;
        let dsn = resolve_lasm_db_postgres_dsn_from_file_contents(
            format!("SEC4_RT_LASM_DB_POSTGRES_DSN_FILE `{}`", file_path.display()).as_str(),
            dsn.as_str(),
        )?;
        return Ok(Some(dsn));
    }
    Err("db adapter postgres requires SEC4_RT_LASM_DB_POSTGRES_DSN to be set".to_string())
}

pub(crate) fn lasm_db_records_adapter_label(adapter: LasmDbRecordsAdapter) -> &'static str {
    match adapter {
        LasmDbRecordsAdapter::RecordsLog => "records.log",
        LasmDbRecordsAdapter::Sqlite => "sqlite",
        LasmDbRecordsAdapter::Postgres => "postgres",
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_lasm_db_postgres_dsn_from_file_contents;

    #[test]
    fn postgres_dsn_file_contents_allow_comments_and_blank_lines() {
        let parsed = resolve_lasm_db_postgres_dsn_from_file_contents(
            "dsn-source",
            "\n# comment\npostgres://sec4:sec4dev@127.0.0.1:5432/sec4_local?sslmode=disable\n",
        )
        .expect("dsn contents should parse");
        assert_eq!(
            parsed,
            "postgres://sec4:sec4dev@127.0.0.1:5432/sec4_local?sslmode=disable"
        );
    }

    #[test]
    fn postgres_dsn_file_contents_reject_multiple_non_comment_lines() {
        let error = resolve_lasm_db_postgres_dsn_from_file_contents(
            "dsn-source",
            "postgres://first\npostgres://second\n",
        )
        .expect_err("multiple DSN lines should fail");
        assert!(error.contains("must contain exactly one DSN line"));
    }
}
