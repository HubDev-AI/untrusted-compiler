use crate::lasm_db_adapter_state::LasmDbPostgresTlsMode;
use crate::{LasmDbRecordsAdapter, RunDbAdapter, RunDbPostgresTlsMode};
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
