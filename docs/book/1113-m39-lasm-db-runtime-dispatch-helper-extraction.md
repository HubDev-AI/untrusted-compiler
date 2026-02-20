# 1113 M39 Slice: LASM DB Runtime Dispatch Helper Extraction

This slice continues DB dispatch isolation by moving dispatch-only header helpers out of CLI orchestration.

## What changed

1. Moved `take_lasm_internal_header_value` and `materialize_lasm_internal_header_value` from `compiler/sec4-cli/src/main.rs` into `compiler/sec4-cli/src/lasm_db_runtime_dispatch.rs`.
2. Kept helper behavior unchanged:
   - case-insensitive internal-header lookup + removal
   - request-placeholder materialization only when placeholder tokens are present
3. Updated dispatch module helper calls to use the local implementations, and removed now-unused definitions from `main.rs`.

## Why

After dispatch function extraction (`1112`), these two helpers were still DB-dispatch-specific code left behind in `main.rs`.

Moving them into the dispatch module keeps DB internal-header handling co-located with DB operation materialization logic and further trims orchestration file responsibility.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_sqlite_records_and_query_one_when_db_intrinsics_are_used`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
3. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
