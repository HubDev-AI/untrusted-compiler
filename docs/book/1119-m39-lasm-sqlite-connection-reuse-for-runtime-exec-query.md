# 1119 M39 Slice: LASM SQLite Connection Reuse For Runtime Exec/Query

This slice removes per-request sqlite open/close churn from LASM runtime DB execution paths.

## What changed

1. Updated sqlite runtime helper in `compiler/sec4-cli/src/lasm_db_runtime_sqlite.rs`:
   - replaced per-call connection open with `lasm_dynamic_sqlite_runtime_connection_mut(...)`.
   - runtime execution now reuses `state.db_records_sqlite_connection`.
2. Updated sqlite runtime DB operation signatures to mutable state:
   - `run_lasm_sqlite_exec(...)`
   - `run_lasm_sqlite_exec_tx(...)`
   - `run_lasm_sqlite_query_one(...)`
3. Added reconnect fallback on sqlite exec failure:
   - drops cached connection and retries once on reconnect for non-lock/non-parameter errors.
4. Updated DB dispatch call sites in `compiler/sec4-cli/src/lasm_db_runtime_dispatch.rs` to pass mutable runtime state into sqlite exec/query operations.

## Why

Append persistence was already using a cached sqlite handle, but runtime execution still opened sqlite on every request.

Reusing a single sqlite connection for both persistence and runtime execution lowers repeated filesystem/open overhead in the hot path while preserving deterministic error behavior.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_sqlite_records_and_query_one_when_db_intrinsics_are_used`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
3. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
