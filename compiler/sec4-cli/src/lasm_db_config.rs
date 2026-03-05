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

fn expand_tilde_in_path(raw: &str) -> PathBuf {
    if !raw.starts_with('~') {
        return PathBuf::from(raw);
    }

    let rest = match raw.strip_prefix("~/") {
        Some(rest) => rest,
        None => return PathBuf::from(raw),
    };

    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .ok();

    match home {
        Some(home_path) => home_path.join(rest),
        None => PathBuf::from(raw),
    }
}

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
        let first_value = found[0].1.as_str();
        if found.iter().all(|(_name, value)| value == first_value) {
            return Ok(Some(first_value.to_string()));
        }
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
    let mut configured = Vec::<(&'static str, String)>::new();

    for &name in keys {
        let raw = match std::env::var(name) {
            Ok(raw) => raw,
            Err(_) => continue,
        };
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }
        configured.push((name, trimmed.to_string()));
    }

    let mut resolved = Vec::<(&'static str, PathBuf)>::new();
    for (name, raw_path) in configured {
        let mut file_path = expand_tilde_in_path(raw_path.as_str());
        if file_path.is_relative() && !file_path.exists() {
            if let Some(project_path) = project_path {
                let candidate = project_path.join(file_path.as_path());
                if candidate.exists() {
                    file_path = candidate;
                }
            }
        }
        if !file_path.exists() {
            return Err(format!(
                "{source_name} configured via `{name}` points to missing file `{}`",
                raw_path
            ));
        }
        let normalized_path = fs::canonicalize(&file_path).unwrap_or(file_path);
        resolved.push((name, normalized_path));
    }

    if resolved.is_empty() {
        return Ok(None);
    }

    if resolved.len() > 1 {
        let first_path = &resolved[0].1;
        if resolved
            .iter()
            .all(|(_name, path)| path.as_path() == first_path.as_path())
        {
            return Ok(Some((resolved[0].0, resolved[0].1.clone())));
        }

        let names = resolved
            .iter()
            .map(|(name, _)| format!("{name}"))
            .collect::<Vec<_>>()
            .join(", ");
        return Err(format!(
            "{source_name} is ambiguous: multiple files were configured: {names}"
        ));
    }

    let (name, file_path) = resolved
        .into_iter()
        .next()
        .expect("resolved env file path list is non-empty");
    Ok(Some((name, file_path)))
}

fn parse_env_file_value(
    raw_contents: &str,
    keys: &[&str],
    source: &str,
) -> Result<Option<String>, String> {
    let mut found: Option<(String, String)> = None;
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
            let value = parse_env_value(&rest[sep + 1..]);
            if value.is_empty() {
                return Err(format!("{source} value for `{}` must be non-empty", name));
            }
            if let Some((previous_name, previous_value)) = &found {
                if previous_value != &value {
                    return Err(format!(
                        "{source} contains multiple values for DSN key (`{previous_name}` and `{name}`)"
                    ));
                }
                continue;
            }
            found = Some((name.to_string(), value));
        }
    }
    Ok(found.map(|(_name, value)| value))
}

fn strip_unquoted_comment(raw_value: &str) -> &str {
    let mut in_single = false;
    let mut in_double = false;
    let mut escaped = false;

    for (index, ch) in raw_value.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }

        match ch {
            '\\' if in_double => {
                escaped = true;
            }
            '\'' if !in_double => {
                in_single = !in_single;
            }
            '"' if !in_single => {
                in_double = !in_double;
            }
            '#' if !in_single && !in_double => {
                return raw_value[..index].trim_end();
            }
            _ => {}
        }
    }

    raw_value.trim_end()
}

fn parse_env_value(raw_value: &str) -> String {
    let unquoted = strip_unquoted_comment(raw_value.trim());
    if unquoted.starts_with('"') {
        return parse_quoted_env_value(unquoted, '"');
    }
    if unquoted.starts_with('\'') {
        return parse_quoted_env_value(unquoted, '\'');
    }
    unquoted.to_string()
}

fn parse_quoted_env_value(raw_value: &str, quote: char) -> String {
    let mut parsed = String::new();
    let mut escaped = false;
    let mut in_value = false;

    for (idx, ch) in raw_value.char_indices() {
        if !in_value {
            if ch == quote {
                in_value = true;
                continue;
            }
            return raw_value.to_string();
        }

        if escaped {
            match quote {
                '"' if ch == 'n' => {
                    parsed.push('\n');
                }
                '"' if ch == 'r' => {
                    parsed.push('\r');
                }
                '"' if ch == 't' => {
                    parsed.push('\t');
                }
                _ if quote == '"' && ch == '"' => parsed.push('"'),
                _ if quote == '"' && ch == '\\' => parsed.push('\\'),
                _ => parsed.push(ch),
            }
            escaped = false;
            continue;
        }

        if quote == '"' && ch == '\\' {
            escaped = true;
            continue;
        }

        if ch == quote {
            let after = raw_value[idx + ch.len_utf8()..].trim_end();
            if after.is_empty() {
                return parsed;
            }
            return raw_value.to_string();
        }

        parsed.push(ch);
    }

    raw_value.to_string()
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

pub(crate) fn resolve_lasm_dynamic_store_base(
    project_path: Option<&Path>,
    explicit_db_base: Option<&Path>,
) -> Option<PathBuf> {
    if let Some(base) = explicit_db_base {
        return Some(base.to_path_buf());
    }
    let Some(value) = resolve_env_value(&LASM_DB_DYNAMIC_STORE_BASE_ENV_KEYS) else {
        return project_path.map(|path| path.join(".lasm-db"));
    };
    if value.is_empty() {
        return project_path.map(|path| path.join(".lasm-db"));
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
        let parsed = parse_env_value(trimmed);
        if parsed.is_empty() {
            continue;
        }
        if dsn.is_some() {
            return Err(format!(
                "{source} must contain exactly one DSN line (excluding comments/blank lines)"
            ));
        }
        dsn = Some(parsed);
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
            .unwrap_or_else(|poisoned| poisoned.into_inner());
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
    fn lasm_dynamic_store_base_defaults_to_dot_lasm_db_under_project() {
        let project_path = std::env::temp_dir().join("sec4-lasm-store-base-defaults");
        let resolved = super::resolve_lasm_dynamic_store_base(Some(project_path.as_path()), None)
            .expect("project path should yield a default store base");
        assert_eq!(resolved, project_path.join(".lasm-db"));
    }

    #[test]
    fn lasm_dynamic_store_base_prefers_explicit_and_env_over_default() {
        let project_path = std::env::temp_dir().join("sec4-lasm-store-base-overrides");
        let explicit = std::env::temp_dir().join("sec4-lasm-store-base-explicit");
        let env_value = std::env::temp_dir().join("sec4-lasm-store-base-env");
        let env_value = env_value.to_string_lossy().to_string();

        let env_default =
            super::resolve_lasm_dynamic_store_base(Some(project_path.as_path()), None);
        assert_eq!(env_default, Some(project_path.join(".lasm-db")));

        with_env_vars(
            &[
                ("SEC4_DB_ALPHA_DB_BASE", Some(env_value.as_str())),
                ("SEC4_RT_LASM_DB_BASE", None),
            ],
            || {
                let resolved =
                    super::resolve_lasm_dynamic_store_base(Some(project_path.as_path()), None)
                        .expect("env DB base should resolve");
                assert_eq!(resolved, PathBuf::from(env_value.as_str()));
            },
        );

        let explicit_resolved = super::resolve_lasm_dynamic_store_base(
            Some(project_path.as_path()),
            Some(explicit.as_path()),
        )
        .expect("explicit DB base should resolve");
        assert_eq!(explicit_resolved, explicit);
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

    #[test]
    fn postgres_dsn_env_aliases_with_same_value_resolve() {
        with_env_vars(
            &[
                ("SEC4_DB_ALPHA_DB_POSTGRES_DSN", Some("postgres://shared")),
                ("SEC4_RT_LASM_DB_POSTGRES_DSN", Some("postgres://shared")),
            ],
            || {
                let adapter = super::LasmDbRecordsAdapter::Postgres;
                let resolved = super::resolve_lasm_dynamic_db_postgres_dsn(adapter, None, None)
                    .expect("same DSN aliases should resolve");
                assert_eq!(resolved, Some("postgres://shared".to_string()));
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
    fn postgres_dsn_file_sources_with_same_path_resolve() {
        let dir = std::env::temp_dir().join("sec4-lasm-postgres-dsn-same-path");
        fs::create_dir_all(&dir).expect("temp test directory should be created");
        let dsn_file = dir.join("shared.env");
        write_dsn_file(&dsn_file, "postgres://shared-file\n");

        let dsn_path = dsn_file.to_string_lossy().to_string();
        with_env_vars(
            &[
                ("SEC4_DB_ALPHA_DB_POSTGRES_DSN", None),
                ("SEC4_RT_LASM_DB_POSTGRES_DSN", None),
                ("SEC4_DB_ALPHA_POSTGRES_DSN_FILE", Some(dsn_path.as_str())),
                (
                    "SEC4_RT_LASM_DB_POSTGRES_DSN_FILE_PATH",
                    Some(dsn_path.as_str()),
                ),
            ],
            || {
                let adapter = super::LasmDbRecordsAdapter::Postgres;
                let resolved =
                    super::resolve_lasm_dynamic_db_postgres_dsn(adapter, None, Some(&dir))
                        .expect("matching DSN file aliases should resolve");
                assert_eq!(resolved, Some("postgres://shared-file".to_string()));
            },
        );

        let _ = fs::remove_file(&dsn_file);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn postgres_dsn_file_source_tilde_path_expands_home() {
        let home = std::env::temp_dir().join("sec4-lasm-postgres-home");
        fs::create_dir_all(&home).expect("temp home directory should be created");
        let home_path = home.to_string_lossy().to_string();
        let dsn_path = home.join(".sec4-postgres.env");
        write_dsn_file(&dsn_path, "postgres://tilde-home\n");
        let tilde_ref = "~/.sec4-postgres.env";

        with_env_vars(
            &[
                ("HOME", Some(home_path.as_str())),
                ("SEC4_DB_ALPHA_DB_POSTGRES_DSN", None),
                ("SEC4_RT_LASM_DB_POSTGRES_DSN", None),
                ("SEC4_DB_ALPHA_POSTGRES_DSN_FILE", Some(tilde_ref)),
                ("SEC4_RT_LASM_DB_POSTGRES_DSN_FILE", None),
                ("SEC4_DB_ALPHA_POSTGRES_DSN_FILE_PATH", None),
                ("SEC4_RT_LASM_DB_POSTGRES_DSN_FILE_PATH", None),
            ],
            || {
                let adapter = super::LasmDbRecordsAdapter::Postgres;
                let resolved = super::resolve_lasm_dynamic_db_postgres_dsn(adapter, None, None)
                    .expect("tilde DSN path should resolve");
                assert_eq!(resolved, Some("postgres://tilde-home".to_string()));
            },
        );

        let _ = fs::remove_file(&dsn_path);
        let _ = fs::remove_dir_all(&home);
    }

    #[test]
    fn postgres_dsn_file_source_missing_file_fails_fast() {
        let dir = std::env::temp_dir().join("sec4-lasm-postgres-dsn-missing-file");
        fs::create_dir_all(&dir).expect("temp test directory should be created");
        let missing_path = dir.join("missing-postgres.env");
        if fs::metadata(&missing_path).is_ok() {
            let _ = fs::remove_file(&missing_path);
        }
        let missing = missing_path.to_string_lossy().to_string();
        with_env_vars(
            &[
                ("SEC4_DB_ALPHA_DB_POSTGRES_DSN", None),
                ("SEC4_RT_LASM_DB_POSTGRES_DSN", None),
                ("SEC4_DB_ALPHA_POSTGRES_DSN_FILE", Some(missing.as_str())),
                ("SEC4_RT_LASM_DB_POSTGRES_DSN_FILE", None),
                ("SEC4_DB_ALPHA_POSTGRES_DSN_FILE_PATH", None),
                ("SEC4_RT_LASM_DB_POSTGRES_DSN_FILE_PATH", None),
            ],
            || {
                let adapter = super::LasmDbRecordsAdapter::Postgres;
                let err = super::resolve_lasm_dynamic_db_postgres_dsn(adapter, None, Some(&dir))
                    .expect_err("missing DSN file must fail");
                assert!(
                    err.contains(
                        "configured via `SEC4_DB_ALPHA_POSTGRES_DSN_FILE` points to missing file"
                    ),
                    "wrong error: {err}"
                );
            },
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn postgres_runtime_env_file_source_missing_file_fails_fast() {
        let dir = std::env::temp_dir().join("sec4-lasm-postgres-runtime-env-missing-file");
        fs::create_dir_all(&dir).expect("temp test directory should be created");
        let missing_path = dir.join("missing-runtime-postgres.env");
        let missing = missing_path.to_string_lossy().to_string();
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
                    Some(missing.as_str()),
                ),
                ("SEC4_RT_LASM_DB_POSTGRES_RUNTIME_ENV_FILE", None),
            ],
            || {
                let adapter = super::LasmDbRecordsAdapter::Postgres;
                let err = super::resolve_lasm_dynamic_db_postgres_dsn(adapter, None, Some(&dir))
                    .expect_err("missing runtime env file must fail");
                assert!(
                    err.contains(
                        "configured via `SEC4_DB_ALPHA_POSTGRES_RUNTIME_ENV_FILE` points to missing file"
                    ),
                    "wrong error: {err}"
                );
            },
        );
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
    fn postgres_dsn_file_contents_supports_inline_comments_and_quoted_hash_values() {
        let parsed = resolve_lasm_db_postgres_dsn_from_file_contents(
            "dsn-source",
            " \"postgres://sec4:sec4dev@127.0.0.1:5432/sec4_local?sslmode=disable#frag\" # trailing comment\n",
        )
        .expect("dsn file contents should parse");
        assert_eq!(
            parsed,
            "postgres://sec4:sec4dev@127.0.0.1:5432/sec4_local?sslmode=disable#frag"
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

    #[test]
    fn postgres_dsn_runtime_env_file_accepts_duplicate_aliases_when_values_match() {
        let dir = std::env::temp_dir().join("sec4-lasm-postgres-runtime-env-same");
        fs::create_dir_all(&dir).expect("temp test directory should be created");
        let path = dir.join("runtime.env");
        let mut file = File::create(&path).expect("runtime env file must be writable");
        file.write_all(
            b"SEC4_DB_ALPHA_DB_POSTGRES_DSN=postgres://shared\nSEC4_RT_LASM_DB_POSTGRES_DSN=postgres://shared\n",
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
                let resolved =
                    super::resolve_lasm_dynamic_db_postgres_dsn(adapter, None, Some(&dir))
                        .expect("same-value runtime env aliases should resolve");
                assert_eq!(resolved, Some("postgres://shared".to_string()));
            },
        );

        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_env_value_supports_inline_comments_and_quoted_values() {
        let parsed = super::parse_env_file_value(
            r#"SEC4_DB_ALPHA_DB_POSTGRES_DSN="postgres://alpha:secret@127.0.0.1:5432/sec4" # production
EXAMPLE_UNUSED_ENV=postgres://ignored
"#,
            &super::LASM_DB_POSTGRES_DSN_KEYS,
            "inline comment source",
        )
        .expect("env parser should return a DSN value");
        assert_eq!(
            parsed,
            Some("postgres://alpha:secret@127.0.0.1:5432/sec4".to_string())
        );
    }

    #[test]
    fn parse_env_value_keeps_hash_symbols_inside_quotes() {
        let parsed = super::parse_env_file_value(
            "SEC4_DB_ALPHA_DB_POSTGRES_DSN='postgres://alpha:secret@127.0.0.1:5432/db#frag'\n",
            &super::LASM_DB_POSTGRES_DSN_KEYS,
            "hash in value source",
        )
        .expect("env parser should preserve hash in quoted DSN");
        assert_eq!(
            parsed,
            Some("postgres://alpha:secret@127.0.0.1:5432/db#frag".to_string())
        );
    }

    #[test]
    fn parse_env_value_supports_escaped_characters_in_double_quotes() {
        let parsed = super::parse_env_file_value(
            r#"SEC4_DB_ALPHA_DB_POSTGRES_DSN="postgres://alpha:\"secret\"@127.0.0.1:5432/db\n\t""#,
            &super::LASM_DB_POSTGRES_DSN_KEYS,
            "escaped env source",
        )
        .expect("env parser should unescape quoted DSN values");
        assert_eq!(
            parsed,
            Some("postgres://alpha:\"secret\"@127.0.0.1:5432/db\n\t".to_string())
        );
    }
}
