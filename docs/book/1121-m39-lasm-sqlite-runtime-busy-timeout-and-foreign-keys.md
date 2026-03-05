# 1121 M39 Slice: LASM SQLite Runtime Busy Timeout And Foreign Keys

This slice hardens sqlite runtime connection defaults used by LASM DB adapters.

## What changed

1. Updated sqlite connect bootstrap in `compiler/sec4-cli/src/lasm_db_adapter_state.rs`:
   - `connect_lasm_dynamic_db_records_sqlite(...)` now sets sqlite busy timeout on every connection.
2. Added env-configurable timeout for sqlite lock waits:
   - `SEC4_RT_LASM_SQLITE_BUSY_TIMEOUT_MS` (default: `2000`).
   - invalid/non-positive values deterministically fall back to default.
3. Enabled sqlite foreign-key enforcement on runtime connections:
   - connection bootstrap now executes `PRAGMA foreign_keys = ON;`.
4. Behavior applies to both runtime DB execution and append persistence paths:
   - both paths already route through the shared sqlite connect helper.

## Why

With real sqlite execution in LASM runtime paths, immediate lock failures are too brittle under concurrent load/restarts.

Busy-timeout and foreign-key defaults make sqlite behavior closer to production DB-client expectations while keeping deterministic runtime configuration and error mapping.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_sqlite_records_and_query_one_when_db_intrinsics_are_used`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
3. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
