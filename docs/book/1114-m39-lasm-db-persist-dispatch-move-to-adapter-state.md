# 1114 M39 Slice: LASM DB Persist Dispatch Move To Adapter-State Module

This slice moves DB-record persistence dispatch wiring out of CLI orchestration.

## What changed

1. Added `persist_lasm_dynamic_db_records_to_disk` to `compiler/sec4-cli/src/lasm_db_adapter_state.rs`.
2. Moved adapter dispatch match (`records-log` / `sqlite` / `postgres`) from `main.rs` into adapter-state module.
3. Updated `compiler/sec4-cli/src/lasm_db_runtime_dispatch.rs` to call `persist_lasm_dynamic_db_records_to_disk` from adapter-state module.
4. Removed now-unused persistence-dispatch helper and imports from `main.rs`.

## Why

Persistence dispatch is adapter-state behavior and no longer belongs in CLI orchestration.

Moving it into adapter-state keeps runtime dispatch + adapter persistence boundaries explicit and reduces DB adapter coupling in `main.rs`.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_sqlite_records_and_query_one_when_db_intrinsics_are_used`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
3. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
