# 1502 M39 Slice: Run DB Postgres DSN-File Project-Relative Resolution

## What changed

1. Extended `validate_and_resolve_run_db_cli_options(...)` in `compiler/sec4-cli/src/lasm_db_cli.rs` to resolve relative `--db-postgres-dsn-file` paths against the selected project root (`--path`) when the cwd-relative path is missing.
2. Updated `cmd_run` to pass `path` into DB CLI option resolution.
3. Added command integration coverage:
   - `run_command_resolves_relative_db_postgres_dsn_file_from_project_path`

## Why

Operators often keep DSN files inside project directories. Before this slice, `--db-postgres-dsn-file` was resolved from process cwd only, which forced absolute paths or manual cwd management. Project-relative fallback keeps behavior deterministic while improving operator ergonomics.

## Validation

1. `cargo test -p sec4 --test commands run_command_resolves_relative_db_postgres_dsn_file_from_project_path`
2. `cargo test -p sec4 --test commands run_command_rejects_db_`
3. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
