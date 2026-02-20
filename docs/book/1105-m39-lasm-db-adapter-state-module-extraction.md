# 1105 M39 Slice: LASM DB Adapter-State Module Extraction

This slice continues DB adapter-layer extraction by moving sqlite/postgres state bootstrap helpers out of `main.rs`.

## What changed

1. Added module `compiler/sec4-cli/src/lasm_db_adapter_state.rs`.
2. Moved adapter-state load/bootstrap helpers from `main.rs`:
   - `load_lasm_dynamic_db_records_from_sqlite`
   - `connect_lasm_dynamic_db_records_postgres`
   - `ensure_lasm_dynamic_db_records_postgres_schema`
   - `load_lasm_dynamic_db_records_from_postgres`
3. Updated `main.rs` to import these helpers from the module.
4. Kept runtime behavior unchanged for:
   - sqlite boot/load path
   - postgres connect/schema/load path
   - record shaping and deterministic error messages

## Why

DB adapter initialization code is storage-specific logic and should be isolated from command/runtime orchestration.

This split creates cleaner seams for future extraction of sqlite/postgres execution paths and package-level adapter boundaries.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_sqlite_records_and_query_one_when_db_intrinsics_are_used`
