# 1116 M39 Slice: LASM DB Plan Type Extraction

This slice moves DB plan model types into the DB plan module.

## What changed

1. Moved DB plan data types from `main.rs` to `compiler/sec4-cli/src/lasm_db_plan.rs`:
   - `LasmSqlQueryPlan`
   - `LasmDbTxPlan`
   - `LasmDbOperationPlan`
2. Kept type shapes/fields unchanged and exposed them as `pub(crate)` in the module.
3. Removed the duplicated type definitions from `main.rs`.

## Why

After extracting DB plan behavior (`1115`), the remaining DB plan model types still lived in CLI orchestration.

Keeping both DB plan logic and DB plan types in the same module completes that boundary and further reduces DB-specific surface area in `main.rs`.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_sqlite_records_and_query_one_when_db_intrinsics_are_used`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
3. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
