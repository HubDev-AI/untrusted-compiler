# 1110 M39 Slice: LASM DB SQLite Schema Helper Module Extraction

This slice removes another DB adapter dependency from CLI orchestration code.

## What changed

1. Moved `ensure_lasm_dynamic_db_records_sqlite_schema` from `compiler/sec4-cli/src/main.rs` to `compiler/sec4-cli/src/lasm_db_adapter_state.rs`.
2. Updated sqlite runtime module imports to consume the schema helper from adapter-state module.
3. Removed now-unused `rusqlite::Connection` import from `main.rs`.

## Why

The sqlite schema bootstrap helper is adapter-state behavior and belongs with adapter-state code, not in top-level command/runtime orchestration.

This extraction further reduces DB coupling in `main.rs` and keeps adapter/runtime modules self-contained.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_sqlite_records_and_query_one_when_db_intrinsics_are_used`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
