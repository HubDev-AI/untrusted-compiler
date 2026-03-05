# 1122 M39 Slice: LASM Postgres Runtime Timeout Defaults

This slice hardens LASM Postgres runtime sessions with deterministic timeout defaults.

## What changed

1. Updated Postgres connect bootstrap in `compiler/sec4-cli/src/lasm_db_adapter_state.rs`:
   - `connect_lasm_dynamic_db_records_postgres(...)` now configures session timeouts immediately after connect.
2. Added env-configurable Postgres runtime timeout controls:
   - `SEC4_RT_LASM_DB_POSTGRES_STATEMENT_TIMEOUT_MS` (default `5000`)
   - `SEC4_RT_LASM_DB_POSTGRES_LOCK_TIMEOUT_MS` (default `2000`)
3. Applied deterministic fallback behavior for invalid/non-positive env values:
   - runtime falls back to default timeout values.
4. Timeout settings are applied via session SQL:
   - `SET statement_timeout = ...`
   - `SET lock_timeout = ...`

## Why

Without bounded statement/lock timeouts, long-running or lock-blocked Postgres operations can stall runtime DB paths under load.

Applying deterministic session timeout defaults makes the Postgres adapter behavior more production-safe while keeping configuration simple and explicit.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_sqlite_records_and_query_one_when_db_intrinsics_are_used`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
3. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
