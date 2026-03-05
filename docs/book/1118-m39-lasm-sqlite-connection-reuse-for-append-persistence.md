# 1118 M39 Slice: LASM SQLite Connection Reuse For Append Persistence

This slice removes per-operation sqlite open/close churn from the LASM DB append hot path.

## What changed

1. Added sqlite connection bootstrap helper in `compiler/sec4-cli/src/lasm_db_adapter_state.rs`:
   - `connect_lasm_dynamic_db_records_sqlite(path)`
   - creates parent directory, opens sqlite file, and ensures schema.
2. Extended `LasmDynamicResponseState` with cached sqlite handle:
   - `db_records_sqlite_connection: Option<rusqlite::Connection>`
3. Updated runtime state bootstrap in `build_lasm_dynamic_response_state(...)`:
   - sqlite adapter now pre-opens the sqlite connection once and stores it in state.
4. Updated sqlite append persistence path:
   - `persist_lasm_dynamic_db_record_append_to_sqlite(...)` now reuses cached connection instead of opening sqlite per record append.
   - on append failure, cached connection is dropped and deterministic full-sync fallback is still executed.

## Why

Even with incremental record persistence, reopening sqlite on every write still adds avoidable overhead.

Caching one sqlite connection per LASM runtime instance keeps append persistence on a lower-latency path while preserving deterministic fallback behavior when the connection becomes invalid.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_sqlite_records_and_query_one_when_db_intrinsics_are_used`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
3. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
