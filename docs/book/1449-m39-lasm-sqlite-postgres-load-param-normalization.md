# M39: LASM SQLite/Postgres Load Param Normalization

Date: 2026-02-22  
Milestone: M39 (adapter history normalization consistency)

## What Changed

- Updated SQLite/Postgres DB-record load paths in `lasm_db_adapter_state.rs`:
  - loaded `params` values are now normalized through canonical DB param normalization.
- Added focused helper coverage in adapter-state tests:
  - canonical JSON normalization,
  - non-JSON passthrough behavior.

## Why

After introducing canonical DB param normalization, adapter-backed persisted histories could still contain older non-canonical param formatting. Normalizing at load keeps runtime history consistent and deterministic across adapters and restarts.

## Result

- SQLite/Postgres-loaded records now use deterministic normalized `params`.
- Runtime history formatting is aligned across records-log/sqlite/postgres loaders.
- Existing DB behavior remains unchanged beyond deterministic normalization.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 normalize_loaded_params`
- `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
