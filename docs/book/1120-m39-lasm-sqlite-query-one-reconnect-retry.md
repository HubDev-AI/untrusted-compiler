# 1120 M39 Slice: LASM SQLite QueryOne Reconnect Retry

This slice hardens LASM sqlite runtime reads by reusing the cached sqlite connection with deterministic reconnect retry behavior.

## What changed

1. Added sqlite runtime retry helper in `compiler/sec4-cli/src/lasm_db_runtime_sqlite.rs`:
   - `run_lasm_sqlite_with_connection_retry(...)`
   - centralizes cached-connection use + one reconnect retry path.
2. Added shared no-retry classifier for sqlite runtime errors:
   - `lasm_sqlite_runtime_error_is_no_retry(...)`
   - keeps lock/parameter failures deterministic without reconnect attempts.
3. Updated `run_lasm_sqlite_query_one(...)` to execute through the retry helper:
   - uses cached connection from runtime state,
   - drops stale connection and retries once for reconnect-eligible failures.
4. Updated `run_lasm_sqlite_exec(...)` to use the same retry helper:
   - keeps existing behavior while removing duplicated reconnect code.

## Why

`queryOne` previously reused the cached connection but did not retry on stale connection failures. This made runtime behavior less resilient than exec paths under long-lived process/database restarts.

Using one shared retry helper keeps sqlite runtime behavior consistent across write/read operations while preserving deterministic non-retry validation/lock errors.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_sqlite_records_and_query_one_when_db_intrinsics_are_used`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
3. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
