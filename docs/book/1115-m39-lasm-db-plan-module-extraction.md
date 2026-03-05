# 1115 M39 Slice: LASM DB Plan Module Extraction

This slice extracts DB operation-plan analysis from CLI orchestration.

## What changed

1. Added module `compiler/sec4-cli/src/lasm_db_plan.rs`.
2. Moved DB plan extraction + header materialization logic from `main.rs` into the module:
   - `extract_lasm_db_operation` and all recursive helper walkers/binders
   - DB-call classifiers (`db.exec`, `db.tx`, `db.execTx`, `db.queryOne`, `sql.q`, `DbCap`)
   - DB operation header-plan applicator (`apply_lasm_db_operation_plan_headers`)
3. Updated `collect_lasm_route_plans` to call `lasm_db_plan::{extract_lasm_db_operation, apply_lasm_db_operation_plan_headers}`.
4. Removed the extracted DB plan block from `main.rs`.

## Why

DB operation planning is compiler/runtime prep logic specific to LASM route extraction and should not stay embedded inside top-level CLI orchestration.

This extraction isolates DB AST planning behavior behind a dedicated module boundary and significantly reduces DB-specific monolith code in `main.rs`.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_sqlite_records_and_query_one_when_db_intrinsics_are_used`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
3. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
