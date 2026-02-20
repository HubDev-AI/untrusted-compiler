# 1104 M39 Slice: LASM DB Records-Log Module Extraction

This slice continues adapter-layer extraction by moving records-log storage helpers out of the CLI monolith.

## What changed

1. Added module `compiler/sec4-cli/src/lasm_db_records_log.rs`.
2. Moved LASM records-log persistence and serialization helpers from `main.rs`:
   - `load_lasm_dynamic_db_records_from_disk`
   - `persist_lasm_dynamic_db_records_to_records_log`
   - `lasm_db_record_to_json`
   - internal `lasm_db_record_from_json`
3. Updated `main.rs` to call module helpers while keeping sqlite/postgres paths unchanged.
4. Kept runtime contracts unchanged:
   - records-log file format
   - deterministic parse warnings
   - `DbListRecordsResponse` record shape/content

## Why

Records-log behavior is adapter-specific storage logic and should not live in the same monolithic file as command dispatch/runtime orchestration.

This module split shrinks `main.rs` and creates a cleaner boundary for future extraction of sqlite/postgres storage layers.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
2. `cargo test -p sec4 --test commands db_max_tx_handles`
