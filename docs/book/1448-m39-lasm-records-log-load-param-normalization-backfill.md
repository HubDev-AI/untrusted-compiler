# M39: LASM Records-Log Load Param Normalization Backfill

Date: 2026-02-22  
Milestone: M39 (records-log signature compatibility)

## What Changed

- Updated `lasm_db_record_from_json` in `lasm_db_records_log.rs`:
  - persisted `params` is now normalized via `normalize_lasm_db_params` on load.
- Added focused load-path regression test:
  - `load_records_log_normalizes_json_params_for_signature_stability`.

## Why

Canonical JSON normalization was added for runtime-generated DB params. Without load-time backfill, older persisted records (created before canonical normalization) could keep non-canonical `params` strings and miss deterministic records-log signature matches after process restart.

## Result

- Persisted records from older formatting styles are normalized at load time.
- Signature lookup behavior remains deterministic across restarts and mixed-history artifacts.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 load_records_log_normalizes_json_params_for_signature_stability`
- `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
