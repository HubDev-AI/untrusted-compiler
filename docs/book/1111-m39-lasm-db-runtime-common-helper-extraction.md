# 1111 M39 Slice: LASM DB Runtime Common Helper Extraction

This slice finishes moving shared DB runtime helpers out of `main.rs`.

## What changed

1. Moved three shared DB runtime helpers from `compiler/sec4-cli/src/main.rs` into `compiler/sec4-cli/src/lasm_db_runtime_common.rs`:
   - `parse_lasm_positive_i64`
   - `is_lasm_valid_db_cap_handle`
   - `normalize_lasm_db_params`
2. Updated `main.rs` imports to use the helpers from `lasm_db_runtime_common`.
3. Removed the original duplicated helper definitions from `main.rs`.

## Why

The helper functions are used by both adapter/runtime flows and are not CLI-orchestration-specific.

Moving them into the runtime-common module keeps DB runtime logic centralized and further reduces DB control-path coupling inside `main.rs`.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_sqlite_records_and_query_one_when_db_intrinsics_are_used`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
