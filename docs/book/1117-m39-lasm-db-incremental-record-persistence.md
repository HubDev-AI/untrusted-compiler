# 1117 M39 Slice: LASM DB Incremental Record Persistence

This slice replaces per-request full DB record-history rewrites with incremental append/upsert persistence for sqlite/postgres adapters.

## What changed

1. Added append persistence helpers in `compiler/sec4-cli/src/lasm_db_adapter_state.rs`:
   - `persist_lasm_dynamic_db_record_append_to_sqlite`
   - `persist_lasm_dynamic_db_record_append_to_postgres`
   - adapter-dispatch wrapper `persist_lasm_dynamic_db_record_append`
2. Updated `compiler/sec4-cli/src/lasm_db_runtime_dispatch.rs` to call append persistence after each DB record push, instead of full-history rewrite dispatch.
3. Added deterministic full-sync fallback behavior:
   - sqlite append failure falls back to full sqlite sync with explicit error chaining
   - postgres append failure falls back to full postgres sync with explicit error chaining
4. Removed unused legacy full-dispatch wrapper `persist_lasm_dynamic_db_records_to_disk`.

## Why

The previous runtime path rewrote the entire persisted DB record history on each operation, which scales poorly as record count grows.

Incremental append/upsert persistence keeps behavior deterministic while reducing write-path overhead for sqlite/postgres adapters and improving LASM DB runtime efficiency under sustained request volume.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_sqlite_records_and_query_one_when_db_intrinsics_are_used`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
3. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
