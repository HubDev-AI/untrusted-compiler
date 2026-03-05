# M39: LASM DB Parameter Canonical JSON Normalization

Date: 2026-02-22  
Milestone: M39 (records-log signature determinism)

## What Changed

- Upgraded `normalize_lasm_db_params` in `lasm_db_runtime_common.rs`:
  - empty input still normalizes to `"0"`,
  - non-JSON text still passes through unchanged,
  - JSON payloads now normalize through recursive canonicalization with deterministic object-key ordering.
- Added focused unit coverage for:
  - top-level object-key ordering,
  - nested object canonicalization,
  - non-JSON passthrough behavior.

## Why

Records-log matching uses normalized DB params as part of signature keys. With trim-only normalization, semantically equivalent JSON payloads could produce different signatures when key order differed.

## Result

- Equivalent JSON parameter payloads now normalize to stable canonical strings.
- Records-log signature lookup is more deterministic across AI-generated payload formatting variations.
- Existing non-JSON parameter behavior remains unchanged.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 normalize_db_params`
- `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
