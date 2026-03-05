# 1106 M39 Slice: LASM DB Adapter Persist Module Extraction

This slice extends adapter-state modularization by moving sqlite/postgres DB record persistence helpers out of `main.rs`.

## What changed

1. Extended `compiler/sec4-cli/src/lasm_db_adapter_state.rs` with:
   - `persist_lasm_dynamic_db_records_to_sqlite`
   - `persist_lasm_dynamic_db_records_to_postgres`
2. Updated `main.rs` to use the module functions from `persist_lasm_dynamic_db_records_to_disk`.
3. Kept existing write-path behavior unchanged:
   - sqlite transaction + replace-all persistence
   - postgres transaction + replace-all persistence
   - postgres reconnect retry on closed/broken-pipe scenarios
   - deterministic persistence error diagnostics

## Why

With both load/bootstrap and persist helpers in the same adapter-state module, sqlite/postgres adapter mechanics are now largely isolated from CLI/runtime orchestration.

This reduces `main.rs` coupling and prepares the next extraction step (runtime execution helpers) with lower risk.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_sqlite_records_and_query_one_when_db_intrinsics_are_used`
