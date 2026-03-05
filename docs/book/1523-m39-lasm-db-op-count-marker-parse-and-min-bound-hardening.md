# 1523 M39 Slice: LASM DB Op-Count Marker Parse and Min-Bound Hardening

This slice hardens LASM internal DB operation-sequence marker handling in runtime dispatch.

## What changed

1. Hardened operation-count marker parsing:
   - `apply_lasm_internal_db_operation_materialization` now validates `X-Sec4-Internal-Db-Op-Count` strictly when present.
   - invalid/non-numeric marker values now return deterministic:
     - status `400`
     - code `DB.OPERATION_INVALID`
     - message: `invalid internal db operation sequence marker`.
2. Added minimum bound validation for operation-count marker:
   - when marker is present, value must be `>= 2`.
   - marker values `0` or `1` now return deterministic:
     - status `400`
     - code `DB.OPERATION_INVALID`
     - message: `internal db operation sequence marker value must be >= 2`.
3. Added focused runtime unit coverage:
   - `rejects_invalid_internal_db_operation_count_marker`
   - `rejects_single_value_internal_db_operation_count_marker`

## Why

Previously invalid operation-count marker values were silently treated as absent and could bypass intended sequence validation paths. Strict parsing and minimum-bound checks keep internal DB marker contracts fail-fast and deterministic.

## Validation

1. `cargo test -p sec4 lasm_db_runtime_dispatch::tests::`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`

