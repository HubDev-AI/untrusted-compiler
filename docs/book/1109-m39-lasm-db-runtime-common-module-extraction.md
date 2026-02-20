# 1109 M39 Slice: LASM DB Runtime Common Module Extraction

This slice factors shared DB runtime utilities out of `main.rs`.

## What changed

1. Added module `compiler/sec4-cli/src/lasm_db_runtime_common.rs`.
2. Moved shared runtime utilities from `main.rs`:
   - `classify_lasm_db_runtime_error`
   - `allocate_lasm_db_tx_handle`
   - `lasm_dynamic_postgres_client_mut`
   - `reconnect_lasm_dynamic_postgres_client`
3. Kept runtime call sites in `main.rs` wired through module imports.
4. Preserved existing behavior:
   - deterministic DB error classification (`validation` / `conflict` / `missing_dependency`)
   - tx-handle capacity/allocator semantics
   - postgres reconnect behavior using existing adapter-state bootstrap

## Why

After sqlite/postgres runtime function extraction, shared DB runtime control helpers remained in `main.rs`.

Moving them into a common runtime module further clarifies DB runtime boundaries and reduces monolithic control flow in command orchestration code.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_sqlite_records_and_query_one_when_db_intrinsics_are_used`
3. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
