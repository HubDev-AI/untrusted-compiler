# 1506 M39 Slice: Env Postgres DSN-File Project-Relative Resolution

## What changed

1. Extended `resolve_lasm_dynamic_db_postgres_dsn(...)` to accept optional project root context and resolve `SEC4_RT_LASM_DB_POSTGRES_DSN_FILE=<relative>` against that root when cwd-relative lookup does not exist.
2. Updated LASM dynamic-state bootstrap to pass selected project root into DB DSN resolver:
   - `sec4 run` path,
   - `sec4 lasm-smoke` path.
3. Added command integration test `run_command_resolves_relative_postgres_dsn_file_env_from_project_path`.

## Why

CLI flag `--db-postgres-dsn-file <relative>` already resolved from project root, but env-based DSN-file fallback still depended on current process working directory. That made operator flows inconsistent across run locations and wrappers.

## Validation

1. `cargo test -p sec4 --test commands run_command_resolves_relative_postgres_dsn_file_env_from_project_path`
2. `cargo test -p sec4 --test commands run_command_rejects_postgres_adapter_with_empty_dsn_file_env`
3. `cargo test -p sec4 --test commands run_command_resolves_relative_db_postgres_dsn_file_from_project_path`
