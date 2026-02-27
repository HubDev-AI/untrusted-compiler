use crate::lasm_db_adapter_state::LasmDbPostgresTlsMode;
use crate::lasm_db_adapter_state::{
    normalize_lasm_db_sqlite_journal_mode, normalize_lasm_db_sqlite_synchronous,
};
use crate::lasm_db_config::load_lasm_db_postgres_dsn_from_file;
use crate::{
    LasmDbRecordsAdapter, RunBackend, RunDbAdapter, RunDbPostgresPersistQueueFullMode,
    RunDbPostgresTlsMode,
};
use std::path::Path;
use std::process::Command;

fn run_db_adapter_arg_value(adapter: RunDbAdapter) -> &'static str {
    match adapter {
        RunDbAdapter::RecordsLog => "records-log",
        RunDbAdapter::Sqlite => "sqlite",
        RunDbAdapter::Postgres => "postgres",
    }
}

pub(crate) fn run_db_adapter_to_lasm_db_records_adapter(
    adapter: RunDbAdapter,
) -> LasmDbRecordsAdapter {
    match adapter {
        RunDbAdapter::RecordsLog => LasmDbRecordsAdapter::RecordsLog,
        RunDbAdapter::Sqlite => LasmDbRecordsAdapter::Sqlite,
        RunDbAdapter::Postgres => LasmDbRecordsAdapter::Postgres,
    }
}

pub(crate) fn push_optional_db_adapter_run_arg(cmd: &mut Command, value: Option<RunDbAdapter>) {
    if let Some(value) = value {
        cmd.arg("--db-adapter").arg(run_db_adapter_arg_value(value));
    }
}

fn run_db_postgres_tls_mode_arg_value(mode: RunDbPostgresTlsMode) -> &'static str {
    match mode {
        RunDbPostgresTlsMode::Auto => "auto",
        RunDbPostgresTlsMode::Disable => "disable",
        RunDbPostgresTlsMode::Require => "require",
    }
}

pub(crate) fn run_db_postgres_tls_mode_to_lasm_db_postgres_tls_mode(
    mode: RunDbPostgresTlsMode,
) -> LasmDbPostgresTlsMode {
    match mode {
        RunDbPostgresTlsMode::Auto => LasmDbPostgresTlsMode::Auto,
        RunDbPostgresTlsMode::Disable => LasmDbPostgresTlsMode::Disable,
        RunDbPostgresTlsMode::Require => LasmDbPostgresTlsMode::Require,
    }
}

pub(crate) fn push_optional_db_postgres_tls_mode_run_arg(
    cmd: &mut Command,
    value: Option<RunDbPostgresTlsMode>,
) {
    if let Some(value) = value {
        cmd.arg("--db-postgres-tls-mode")
            .arg(run_db_postgres_tls_mode_arg_value(value));
    }
}

fn run_db_postgres_persist_queue_full_mode_arg_value(
    mode: RunDbPostgresPersistQueueFullMode,
) -> &'static str {
    match mode {
        RunDbPostgresPersistQueueFullMode::Block => "block",
        RunDbPostgresPersistQueueFullMode::SyncFallback => "sync-fallback",
    }
}

pub(crate) fn push_optional_db_postgres_persist_queue_full_mode_run_arg(
    cmd: &mut Command,
    value: Option<RunDbPostgresPersistQueueFullMode>,
) {
    if let Some(value) = value {
        cmd.arg("--db-postgres-persist-queue-full-mode")
            .arg(run_db_postgres_persist_queue_full_mode_arg_value(value));
    }
}

pub(crate) struct ResolvedRunDbCliOptions {
    pub(crate) effective_db_adapter: Option<RunDbAdapter>,
    pub(crate) explicit_db_postgres_dsn: Option<String>,
    pub(crate) explicit_db_sqlite_journal_mode: Option<String>,
    pub(crate) explicit_db_sqlite_synchronous: Option<String>,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn validate_and_resolve_run_db_cli_options(
    project_path: &Path,
    backend: RunBackend,
    db_base: Option<&Path>,
    db_adapter: Option<RunDbAdapter>,
    db_postgres_dsn: Option<&str>,
    db_postgres_dsn_file: Option<&Path>,
    db_postgres_tls_mode: Option<RunDbPostgresTlsMode>,
    db_max_tx_handles: Option<u64>,
    db_records_max: Option<u64>,
    db_query_one_row_max_bytes: Option<u64>,
    db_query_one_row_max_columns: Option<u64>,
    db_sql_template_max_bytes: Option<u64>,
    db_params_max_bytes: Option<u64>,
    db_params_max_entries: Option<u64>,
    db_postgres_statement_cache_max: Option<u64>,
    db_postgres_placeholder_cache_max: Option<u64>,
    db_postgres_statement_timeout_ms: Option<u64>,
    db_postgres_lock_timeout_ms: Option<u64>,
    db_postgres_connect_timeout_ms: Option<u64>,
    db_postgres_persist_batch_max: Option<u64>,
    db_postgres_persist_queue_full_mode: Option<RunDbPostgresPersistQueueFullMode>,
    db_sqlite_busy_timeout_ms: Option<u64>,
    db_sqlite_journal_mode: Option<String>,
    db_sqlite_synchronous: Option<String>,
    db_postgres_retryable_conflict_retry_max: Option<u64>,
    db_sqlite_lock_retry_max: Option<u64>,
    db_sqlite_lock_retry_delay_ms: Option<u64>,
) -> Result<ResolvedRunDbCliOptions, String> {
    if backend != RunBackend::Lasm && db_base.is_some() {
        return Err("--db-base is only supported with --backend lasm".to_string());
    }
    if backend != RunBackend::Lasm && db_adapter.is_some() {
        return Err("--db-adapter is only supported with --backend lasm".to_string());
    }
    if backend != RunBackend::Lasm && db_postgres_dsn.is_some() {
        return Err("--db-postgres-dsn is only supported with --backend lasm".to_string());
    }
    if backend != RunBackend::Lasm && db_postgres_dsn_file.is_some() {
        return Err("--db-postgres-dsn-file is only supported with --backend lasm".to_string());
    }
    if backend != RunBackend::Lasm && db_postgres_tls_mode.is_some() {
        return Err("--db-postgres-tls-mode is only supported with --backend lasm".to_string());
    }
    if backend != RunBackend::Lasm && db_max_tx_handles.is_some() {
        return Err("--db-max-tx-handles is only supported with --backend lasm".to_string());
    }
    if backend != RunBackend::Lasm && db_records_max.is_some() {
        return Err("--db-records-max is only supported with --backend lasm".to_string());
    }
    if backend != RunBackend::Lasm && db_query_one_row_max_bytes.is_some() {
        return Err(
            "--db-query-one-row-max-bytes is only supported with --backend lasm".to_string(),
        );
    }
    if backend != RunBackend::Lasm && db_query_one_row_max_columns.is_some() {
        return Err(
            "--db-query-one-row-max-columns is only supported with --backend lasm".to_string(),
        );
    }
    if backend != RunBackend::Lasm && db_sql_template_max_bytes.is_some() {
        return Err(
            "--db-sql-template-max-bytes is only supported with --backend lasm".to_string(),
        );
    }
    if backend != RunBackend::Lasm && db_params_max_bytes.is_some() {
        return Err("--db-params-max-bytes is only supported with --backend lasm".to_string());
    }
    if backend != RunBackend::Lasm && db_params_max_entries.is_some() {
        return Err("--db-params-max-entries is only supported with --backend lasm".to_string());
    }
    if backend != RunBackend::Lasm && db_postgres_statement_cache_max.is_some() {
        return Err(
            "--db-postgres-statement-cache-max is only supported with --backend lasm".to_string(),
        );
    }
    if backend != RunBackend::Lasm && db_postgres_placeholder_cache_max.is_some() {
        return Err(
            "--db-postgres-placeholder-cache-max is only supported with --backend lasm".to_string(),
        );
    }
    if backend != RunBackend::Lasm && db_postgres_statement_timeout_ms.is_some() {
        return Err(
            "--db-postgres-statement-timeout-ms is only supported with --backend lasm".to_string(),
        );
    }
    if backend != RunBackend::Lasm && db_postgres_lock_timeout_ms.is_some() {
        return Err(
            "--db-postgres-lock-timeout-ms is only supported with --backend lasm".to_string(),
        );
    }
    if backend != RunBackend::Lasm && db_postgres_connect_timeout_ms.is_some() {
        return Err(
            "--db-postgres-connect-timeout-ms is only supported with --backend lasm".to_string(),
        );
    }
    if backend != RunBackend::Lasm && db_postgres_persist_batch_max.is_some() {
        return Err(
            "--db-postgres-persist-batch-max is only supported with --backend lasm".to_string(),
        );
    }
    if backend != RunBackend::Lasm && db_postgres_persist_queue_full_mode.is_some() {
        return Err(
            "--db-postgres-persist-queue-full-mode is only supported with --backend lasm"
                .to_string(),
        );
    }
    if backend != RunBackend::Lasm && db_sqlite_busy_timeout_ms.is_some() {
        return Err(
            "--db-sqlite-busy-timeout-ms is only supported with --backend lasm".to_string(),
        );
    }
    if backend != RunBackend::Lasm && db_sqlite_journal_mode.is_some() {
        return Err("--db-sqlite-journal-mode is only supported with --backend lasm".to_string());
    }
    if backend != RunBackend::Lasm && db_sqlite_synchronous.is_some() {
        return Err("--db-sqlite-synchronous is only supported with --backend lasm".to_string());
    }
    if backend != RunBackend::Lasm && db_postgres_retryable_conflict_retry_max.is_some() {
        return Err(
            "--db-postgres-retryable-conflict-retry-max is only supported with --backend lasm"
                .to_string(),
        );
    }
    if backend != RunBackend::Lasm && db_sqlite_lock_retry_max.is_some() {
        return Err("--db-sqlite-lock-retry-max is only supported with --backend lasm".to_string());
    }
    if backend != RunBackend::Lasm && db_sqlite_lock_retry_delay_ms.is_some() {
        return Err(
            "--db-sqlite-lock-retry-delay-ms is only supported with --backend lasm".to_string(),
        );
    }

    if db_max_tx_handles == Some(0) {
        return Err("--db-max-tx-handles must be >= 1".to_string());
    }
    if db_records_max == Some(0) {
        return Err("--db-records-max must be >= 1".to_string());
    }
    if db_query_one_row_max_bytes == Some(0) {
        return Err("--db-query-one-row-max-bytes must be >= 1".to_string());
    }
    if db_query_one_row_max_columns == Some(0) {
        return Err("--db-query-one-row-max-columns must be >= 1".to_string());
    }
    if db_sql_template_max_bytes == Some(0) {
        return Err("--db-sql-template-max-bytes must be >= 1".to_string());
    }
    if db_params_max_bytes == Some(0) {
        return Err("--db-params-max-bytes must be >= 1".to_string());
    }
    if db_params_max_entries == Some(0) {
        return Err("--db-params-max-entries must be >= 1".to_string());
    }
    if db_postgres_statement_cache_max == Some(0) {
        return Err("--db-postgres-statement-cache-max must be >= 1".to_string());
    }
    if db_postgres_placeholder_cache_max == Some(0) {
        return Err("--db-postgres-placeholder-cache-max must be >= 1".to_string());
    }
    if db_postgres_statement_timeout_ms == Some(0) {
        return Err("--db-postgres-statement-timeout-ms must be >= 1".to_string());
    }
    if db_postgres_lock_timeout_ms == Some(0) {
        return Err("--db-postgres-lock-timeout-ms must be >= 1".to_string());
    }
    if db_postgres_connect_timeout_ms == Some(0) {
        return Err("--db-postgres-connect-timeout-ms must be >= 1".to_string());
    }
    if db_postgres_persist_batch_max == Some(0) {
        return Err("--db-postgres-persist-batch-max must be >= 1".to_string());
    }
    if db_sqlite_busy_timeout_ms == Some(0) {
        return Err("--db-sqlite-busy-timeout-ms must be >= 1".to_string());
    }
    if db_postgres_dsn.is_some() && db_postgres_dsn_file.is_some() {
        return Err("use only one of --db-postgres-dsn or --db-postgres-dsn-file".to_string());
    }
    if db_query_one_row_max_bytes
        .map(|value| usize::try_from(value).is_err())
        .unwrap_or(false)
    {
        return Err("--db-query-one-row-max-bytes exceeds platform limits".to_string());
    }
    if db_query_one_row_max_columns
        .map(|value| usize::try_from(value).is_err())
        .unwrap_or(false)
    {
        return Err("--db-query-one-row-max-columns exceeds platform limits".to_string());
    }
    if db_sql_template_max_bytes
        .map(|value| usize::try_from(value).is_err())
        .unwrap_or(false)
    {
        return Err("--db-sql-template-max-bytes exceeds platform limits".to_string());
    }
    if db_params_max_bytes
        .map(|value| usize::try_from(value).is_err())
        .unwrap_or(false)
    {
        return Err("--db-params-max-bytes exceeds platform limits".to_string());
    }
    if db_params_max_entries
        .map(|value| usize::try_from(value).is_err())
        .unwrap_or(false)
    {
        return Err("--db-params-max-entries exceeds platform limits".to_string());
    }
    if db_postgres_persist_batch_max
        .map(|value| usize::try_from(value).is_err())
        .unwrap_or(false)
    {
        return Err("--db-postgres-persist-batch-max exceeds platform limits".to_string());
    }

    let explicit_db_sqlite_journal_mode = if let Some(mode) = db_sqlite_journal_mode {
        let trimmed = mode.trim();
        if trimmed.is_empty() {
            return Err("--db-sqlite-journal-mode must not be empty".to_string());
        }
        match normalize_lasm_db_sqlite_journal_mode(trimmed) {
            Some(normalized) => Some(normalized.to_string()),
            None => {
                return Err(
                    "--db-sqlite-journal-mode must be one of wal, delete, truncate, persist, memory, off"
                        .to_string(),
                );
            }
        }
    } else {
        None
    };
    let explicit_db_sqlite_synchronous = if let Some(mode) = db_sqlite_synchronous {
        let trimmed = mode.trim();
        if trimmed.is_empty() {
            return Err("--db-sqlite-synchronous must not be empty".to_string());
        }
        match normalize_lasm_db_sqlite_synchronous(trimmed) {
            Some(normalized) => Some(normalized.to_string()),
            None => {
                return Err(
                    "--db-sqlite-synchronous must be one of off, normal, full, extra".to_string(),
                );
            }
        }
    } else {
        None
    };
    let postgres_runtime_overrides = db_postgres_dsn.is_some()
        || db_postgres_dsn_file.is_some()
        || db_postgres_tls_mode.is_some()
        || db_postgres_statement_cache_max.is_some()
        || db_postgres_placeholder_cache_max.is_some()
        || db_postgres_statement_timeout_ms.is_some()
        || db_postgres_lock_timeout_ms.is_some()
        || db_postgres_connect_timeout_ms.is_some()
        || db_postgres_persist_batch_max.is_some()
        || db_postgres_persist_queue_full_mode.is_some()
        || db_postgres_retryable_conflict_retry_max.is_some();
    let sqlite_runtime_overrides = db_sqlite_busy_timeout_ms.is_some()
        || explicit_db_sqlite_journal_mode.is_some()
        || explicit_db_sqlite_synchronous.is_some()
        || db_sqlite_lock_retry_max.is_some()
        || db_sqlite_lock_retry_delay_ms.is_some();
    if backend == RunBackend::Lasm && postgres_runtime_overrides && sqlite_runtime_overrides {
        return Err(
            "postgres and sqlite runtime overrides cannot be combined in the same run".to_string(),
        );
    }
    let effective_db_adapter = if backend == RunBackend::Lasm && postgres_runtime_overrides {
        match db_adapter {
            Some(RunDbAdapter::Postgres) => Some(RunDbAdapter::Postgres),
            Some(_) => {
                return Err(
                    "postgres DSN/runtime overrides require --db-adapter postgres when adapter is set explicitly".to_string(),
                );
            }
            None => Some(RunDbAdapter::Postgres),
        }
    } else if backend == RunBackend::Lasm && sqlite_runtime_overrides {
        match db_adapter {
            Some(RunDbAdapter::Sqlite) => Some(RunDbAdapter::Sqlite),
            Some(_) => {
                return Err(
                    "sqlite runtime overrides require --db-adapter sqlite when adapter is set explicitly".to_string(),
                );
            }
            None => Some(RunDbAdapter::Sqlite),
        }
    } else {
        db_adapter
    };

    let explicit_db_postgres_dsn = if let Some(dsn) = db_postgres_dsn {
        let dsn = dsn.trim();
        if dsn.is_empty() {
            return Err("--db-postgres-dsn must not be empty".to_string());
        }
        Some(dsn.to_string())
    } else if let Some(path) = db_postgres_dsn_file {
        let mut resolved_path = path.to_path_buf();
        if path.is_relative() && !resolved_path.exists() {
            let candidate = project_path.join(path);
            if candidate.exists() {
                resolved_path = candidate;
            }
        }
        Some(load_lasm_db_postgres_dsn_from_file(
            resolved_path.as_path(),
        )?)
    } else {
        None
    };

    Ok(ResolvedRunDbCliOptions {
        effective_db_adapter,
        explicit_db_postgres_dsn,
        explicit_db_sqlite_journal_mode,
        explicit_db_sqlite_synchronous,
    })
}
