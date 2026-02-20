# 1085 M39 Slice: LASM Postgres DSN File Env Fallback

This slice extends LASM Postgres DSN resolution to support secret-file env fallback via `SEC4_RT_LASM_DB_POSTGRES_DSN_FILE`.

## What changed

1. Extended Postgres DSN resolution precedence:
   1. explicit run flag (`--db-postgres-dsn`)
   2. `SEC4_RT_LASM_DB_POSTGRES_DSN`
   3. `SEC4_RT_LASM_DB_POSTGRES_DSN_FILE`
2. Added deterministic env-file diagnostics:
   - empty `SEC4_RT_LASM_DB_POSTGRES_DSN_FILE` path is rejected
   - unreadable DSN file path is rejected with explicit path diagnostics
   - empty DSN file content is rejected (`must contain a non-empty DSN`)
3. Updated missing-DSN command test to clear both DSN env sources (`SEC4_RT_LASM_DB_POSTGRES_DSN` and `SEC4_RT_LASM_DB_POSTGRES_DSN_FILE`).
4. Added dedicated command coverage for invalid empty DSN file env content.

## Why

Many deployment environments mount secrets as files.  
This fallback allows Postgres-backed LASM runs to consume file-mounted DSNs without forcing inline env values or explicit flags, while keeping deterministic validation behavior.

## Validation

1. `cargo test -p sec4 --test commands run_command_rejects_postgres_adapter_without_dsn`
2. `cargo test -p sec4 --test commands run_command_rejects_postgres_adapter_with_empty_dsn_file_env`
3. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
