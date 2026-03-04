use crate::LasmDbRecordsAdapter;
use std::fs;
use std::path::{Path, PathBuf};

const LASM_DB_TX_MAX_HANDLES_DEFAULT: usize = 256;
const LASM_DB_DYNAMIC_ADAPTER_ENV_KEYS: [&str; 2] =
    ["SEC4_DB_ALPHA_DB_ADAPTER", "SEC4_RT_LASM_DB_ADAPTER"];
const LASM_DB_DYNAMIC_STORE_BASE_ENV_KEYS: [&str; 2] =
    ["SEC4_DB_ALPHA_DB_BASE", "SEC4_RT_LASM_DB_BASE"];
pub(crate) const LASM_DB_POSTGRES_DSN_KEYS: [&str; 2] = [
    "SEC4_DB_ALPHA_DB_POSTGRES_DSN",
    "SEC4_RT_LASM_DB_POSTGRES_DSN",
];
pub(crate) const LASM_DB_POSTGRES_DSN_FILE_KEYS: [&str; 4] = [
    "SEC4_DB_ALPHA_POSTGRES_DSN_FILE",
    "SEC4_RT_LASM_DB_POSTGRES_DSN_FILE",
    "SEC4_DB_ALPHA_POSTGRES_DSN_FILE_PATH",
    "SEC4_RT_LASM_DB_POSTGRES_DSN_FILE_PATH",
];
pub(crate) const LASM_DB_POSTGRES_RUNTIME_ENV_KEYS: [&str; 4] = [
    "SEC4_DB_ALPHA_POSTGRES_RUNTIME_ENV_FILE",
    "SEC4_DB_ALPHA_POSTGRES_RUNTIME_DSN_FILE",
    "SEC4_RT_LASM_DB_POSTGRES_RUNTIME_ENV_FILE",
    "SEC4_RT_LASM_DB_POSTGRES_RUNTIME_DSN_FILE",
];
pub(crate) const LASM_DB_POSTGRES_DSN_CONFIG_ERROR_MESSAGE: &str =
    "db adapter postgres requires --db-postgres-dsn or a DSN source via SEC4_DB_ALPHA_DB_POSTGRES_DSN, SEC4_RT_LASM_DB_POSTGRES_DSN, SEC4_DB_ALPHA_POSTGRES_DSN_FILE/_PATH, SEC4_RT_LASM_DB_POSTGRES_DSN_FILE/_PATH, SEC4_DB_ALPHA_POSTGRES_RUNTIME_ENV_FILE/SEC4_DB_ALPHA_POSTGRES_RUNTIME_DSN_FILE, or SEC4_RT_LASM_DB_POSTGRES_RUNTIME_ENV_FILE/SEC4_RT_LASM_DB_POSTGRES_RUNTIME_DSN_FILE";

fn resolve_env_value(keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|name| {
        std::env::var(name).ok().and_then(|raw| {
            let value = raw.trim();
            if value.is_empty() {
                None
            } else {
                Some(value.to_string())
            }
        })
    })
}

fn resolve_unique_env_value(
    keys: &[&'static str],
    source_name: &str,
) -> Result<Option<String>, String> {
    let mut found = Vec::<(&'static str, String)>::new();

    for &name in keys {
        if let Ok(raw) = std::env::var(name) {
            let value = raw.trim();
            if !value.is_empty() {
                found.push((name, value.to_string()));
            }
        }
    }

    if found.len() > 1 {
        let names = found
            .iter()
            .map(|(name, _)| format!("{name}"))
            .collect::<Vec<_>>()
            .join(", ");
        return Err(format!(
            "{source_name} is ambiguous: multiple values were configured: {names}"
        ));
    }

    Ok(found.into_iter().next().map(|(_, value)| value))
}

fn resolve_unique_env_file_path_with_candidates(
    keys: &[&'static str],
    project_path: Option<&Path>,
    source_name: &str,
) -> Result<Option<(&'static str, PathBuf)>, String> {
    let mut found = Vec::<(&'static str, PathBuf)>::new();

    for &name in keys {
        if let Some(file_path) = resolve_env_file_path(name, project_path) {
            found.push((name, file_path));
        }
    }

    if found.len() > 1 {
        let names = found
            .iter()
            .map(|(name, _)| format!("{name}"))
            .collect::<Vec<_>>()
            .join(", ");
        return Err(format!(
            "{source_name} is ambiguous: multiple files were configured: {names}"
        ));
    }

    Ok(found.into_iter().next())
}

fn resolve_env_file_path(name: &str, project_path: Option<&Path>) -> Option<PathBuf> {
    let raw = std::env::var(name).ok()?;
    let candidate = raw.trim();
    if candidate.is_empty() {
        return None;
    }
    let mut file_path = PathBuf::from(candidate);
    if file_path.is_relative() && !file_path.exists() {
        if let Some(project_path) = project_path {
            let candidate = project_path.join(file_path.as_path());
            if candidate.exists() {
                file_path = candidate;
            }
        }
    }
    if file_path.exists() {
        Some(file_path)
    } else {
        None
    }
}

fn parse_env_file_value(
    raw_contents: &str,
    keys: &[&str],
    source: &str,
) -> Result<Option<String>, String> {
    let mut found: Option<String> = None;
    for line in raw_contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let mut rest = trimmed;
        if let Some(after_export) = trimmed.strip_prefix("export ") {
            rest = after_export.trim();
        }
        if let Some(sep) = rest.find('=') {
            let name = rest[..sep].trim();
            if !keys.iter().any(|key| key == &name) {
                continue;
            }
            let value = rest[sep + 1..].trim().trim_matches('"').trim_matches('\'');
            if value.is_empty() {
                return Err(format!("{source} value for `{}` must be non-empty", name));
            }
            if let Some(previous) = found {
                return Err(format!(
                    "{source} contains multiple values for DSN key (`{previous}` and `{name}`)"
                ));
            }
            found = Some(value.to_string());
        }
    }
    Ok(found)
}

fn parse_first_existing_postgres_dsn_file(
    path: PathBuf,
    file_source: &str,
) -> Result<String, String> {
    let raw = fs::read_to_string(&path)
        .map_err(|err| format!("could not read {file_source} `{}`: {err}", path.display()))?;
    let dsn_source = format!("{file_source} `{}`", path.display());
    resolve_lasm_db_postgres_dsn_from_file_contents(dsn_source.as_str(), raw.as_str())
}

fn parse_runtime_env_file_for_postgres_dsn(
    path: PathBuf,
    keys: &[&str],
    source: &str,
) -> Result<Option<String>, String> {
    let raw = fs::read_to_string(&path).map_err(|err| {
        format!(
            "could not read runtime env file `{}` from {source}: {err}",
            path.display()
        )
    })?;
    parse_env_file_value(
        raw.as_str(),
        keys,
        &format!("runtime env file `{}` from {source}", path.display()),
    )
}

pub(crate) fn resolve_lasm_dynamic_store_base(explicit_db_base: Option<&Path>) -> Option<PathBuf> {
    if let Some(base) = explicit_db_base {
        return Some(base.to_path_buf());
    }
    let Some(value) = resolve_env_value(&LASM_DB_DYNAMIC_STORE_BASE_ENV_KEYS) else {
        return None;
    };
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
    let Some(raw) = resolve_env_value(&LASM_DB_DYNAMIC_ADAPTER_ENV_KEYS) else {
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
        "warning: unsupported SEC4_DB_ALPHA_DB_ADAPTER/SEC4_RT_LASM_DB_ADAPTER value `{}`; defaulting to records.log adapter",
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
                "db adapter postgres requires a non-empty --db-postgres-dsn value".to_string(),
            );
        }
        return Ok(Some(dsn.to_string()));
    }
    if let Some(raw) = resolve_unique_env_value(
        &LASM_DB_POSTGRES_DSN_KEYS,
        "db adapter postgres DSN environment variables",
    )? {
        return Ok(Some(raw));
    }

    if let Some((file_env_name, file_path)) = resolve_unique_env_file_path_with_candidates(
        &LASM_DB_POSTGRES_DSN_FILE_KEYS,
        project_path,
        "db adapter postgres DSN file sources",
    )? {
        return parse_first_existing_postgres_dsn_file(file_path, file_env_name).map(Some);
    }

    if let Some((runtime_env_name, env_file_path)) = resolve_unique_env_file_path_with_candidates(
        &LASM_DB_POSTGRES_RUNTIME_ENV_KEYS,
        project_path,
        "db adapter postgres runtime env file sources",
    )? {
        let dsn = parse_runtime_env_file_for_postgres_dsn(
            env_file_path,
            &LASM_DB_POSTGRES_DSN_KEYS,
            runtime_env_name,
        )?;
        if let Some(dsn_value) = dsn {
            return Ok(Some(dsn_value));
        }
    }
    Err(LASM_DB_POSTGRES_DSN_CONFIG_ERROR_MESSAGE.to_string())
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

    use std::env;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::PathBuf;
    use std::sync::Mutex;

    static TEST_ENV_LOCK: Mutex<()> = Mutex::new(());

    fn with_env_vars<R>(vars: &[(&str, Option<&str>)], action: impl FnOnce() -> R) -> R {
        let _env_lock = TEST_ENV_LOCK
            .lock()
            .expect("test environment lock should be available");
        let originals: Vec<(String, Option<String>)> = vars
            .iter()
            .map(|(name, _value)| {
                (
                    (*name).to_string(),
                    env::var(name).ok().map(|old| old.to_string()),
                )
            })
            .collect();

        for (name, value) in vars {
            match value {
                Some(value) => env::set_var(name, value),
                None => env::remove_var(name),
            }
        }

        let result = action();

        for (name, old) in originals {
            match old {
                Some(old) => env::set_var(&name, old),
                None => env::remove_var(name),
            }
        }

        result
    }

    #[test]
    fn postgres_dsn_env_ambiguous_between_aliases_fails() {
        with_env_vars(
            &[
                ("SEC4_DB_ALPHA_DB_POSTGRES_DSN", Some("postgres://alpha")),
                ("SEC4_RT_LASM_DB_POSTGRES_DSN", Some("postgres://lasm")),
            ],
            || {
                let adapter = super::LasmDbRecordsAdapter::Postgres;
                let err = super::resolve_lasm_dynamic_db_postgres_dsn(adapter, None, None)
                    .expect_err("DSN aliases must not be set simultaneously");
                assert!(
                    err.contains("db adapter postgres DSN environment variables is ambiguous"),
                    "wrong error: {err}"
                );
            },
        );
    }

    fn write_dsn_file(path: &PathBuf, raw: &str) {
        let mut file = File::create(path).expect("test DSN file must be writable");
        file.write_all(raw.as_bytes())
            .expect("test DSN file content write must succeed");
    }

    #[test]
    fn postgres_dsn_file_sources_ambiguous_between_aliases_fails() {
        let dir = std::env::temp_dir().join("sec4-lasm-postgres-dsn-conflict");
        fs::create_dir_all(&dir).expect("temp test directory should be created");
        let first = dir.join("alpha.env");
        let second = dir.join("lasm.env");
        write_dsn_file(&first, "postgres://alpha\n");
        write_dsn_file(&second, "postgres://lasm\n");

        let alpha_path = first.to_string_lossy().to_string();
        let lasm_path = second.to_string_lossy().to_string();
        with_env_vars(
            &[
                ("SEC4_DB_ALPHA_DB_POSTGRES_DSN", None),
                ("SEC4_RT_LASM_DB_POSTGRES_DSN", None),
                ("SEC4_DB_ALPHA_POSTGRES_DSN_FILE", Some(alpha_path.as_str())),
                (
                    "SEC4_RT_LASM_DB_POSTGRES_DSN_FILE",
                    Some(lasm_path.as_str()),
                ),
                ("SEC4_DB_ALPHA_POSTGRES_DSN_FILE_PATH", None),
                ("SEC4_RT_LASM_DB_POSTGRES_DSN_FILE_PATH", None),
            ],
            || {
                let adapter = super::LasmDbRecordsAdapter::Postgres;
                let err = super::resolve_lasm_dynamic_db_postgres_dsn(adapter, None, Some(&dir))
                    .expect_err("conflicting DSN files must fail");
                assert!(
                    err.contains("DSN file sources is ambiguous"),
                    "wrong error: {err}"
                );
            },
        );

        let _ = fs::remove_file(&first);
        let _ = fs::remove_file(&second);
        let _ = fs::remove_dir_all(&dir);
    }

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

    #[test]
    fn postgres_dsn_runtime_env_file_rejects_multiple_aliases() {
        let dir = std::env::temp_dir().join("sec4-lasm-postgres-runtime-env-conflict");
        fs::create_dir_all(&dir).expect("temp test directory should be created");
        let path = dir.join("runtime.env");
        let mut file = File::create(&path).expect("runtime env file must be writable");
        file.write_all(
            b"SEC4_DB_ALPHA_DB_POSTGRES_DSN=postgres://alpha\nSEC4_RT_LASM_DB_POSTGRES_DSN=postgres://lasm\n",
        )
        .expect("runtime env file write should succeed");

        with_env_vars(
            &[
                ("SEC4_DB_ALPHA_DB_POSTGRES_DSN", None),
                ("SEC4_RT_LASM_DB_POSTGRES_DSN", None),
                ("SEC4_DB_ALPHA_POSTGRES_DSN_FILE", None),
                ("SEC4_RT_LASM_DB_POSTGRES_DSN_FILE", None),
                ("SEC4_DB_ALPHA_POSTGRES_DSN_FILE_PATH", None),
                ("SEC4_RT_LASM_DB_POSTGRES_DSN_FILE_PATH", None),
                (
                    "SEC4_DB_ALPHA_POSTGRES_RUNTIME_ENV_FILE",
                    Some(path.to_string_lossy().as_ref()),
                ),
            ],
            || {
                let adapter = super::LasmDbRecordsAdapter::Postgres;
                let err = super::resolve_lasm_dynamic_db_postgres_dsn(adapter, None, Some(&dir))
                    .expect_err("multiple runtime env DSN aliases should fail");
                assert!(err.contains("runtime env file `"), "wrong error: {err}");
            },
        );

        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir_all(&dir);
    }
}
