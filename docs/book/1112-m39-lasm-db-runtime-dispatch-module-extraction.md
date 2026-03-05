# 1112 M39 Slice: LASM DB Runtime Dispatch Module Extraction

This slice moves the internal DB operation dispatcher out of CLI orchestration.

## What changed

1. Added module `compiler/sec4-cli/src/lasm_db_runtime_dispatch.rs`.
2. Moved the full `apply_lasm_internal_db_operation_materialization` implementation from `compiler/sec4-cli/src/main.rs` into the new module.
3. Kept `main.rs` on a thin delegator call to `lasm_db_runtime_dispatch::apply_lasm_internal_db_operation_materialization(...)`.
4. Updated module imports so postgres adapter-state/runtime modules consume postgres-client reconnect helpers from `lasm_db_runtime_common` directly, instead of relying on root-scope alias imports.

## Why

The DB materialization/dispatch path was still one of the largest DB runtime blocks in `main.rs`.

Moving it behind a dedicated runtime-dispatch module keeps CLI orchestration thinner and makes DB operation flow (`exec` / `execTx` / `queryOne`) easier to evolve independently from command wiring.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_sqlite_records_and_query_one_when_db_intrinsics_are_used`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
3. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
