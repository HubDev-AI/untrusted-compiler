# 1520 M39 Slice: LASM DB Internal Marker Required Handle/Schema Hardening

This slice hardens LASM internal DB operation dispatch by removing silent fallback defaults for required internal DB markers.

## What changed

1. Hardened `db.exec` marker handling in LASM DB runtime dispatch:
   - `apply_lasm_internal_db_operation_materialization_single` now requires an explicit internal DB handle marker.
   - Missing handle marker now returns deterministic:
     - status `400`
     - code `DB.EXEC_INVALID`
     - existing validation message contract (`db.exec requires db capability and query handle`).
2. Hardened `db.queryOne` marker handling in LASM DB runtime dispatch:
   - runtime now requires explicit internal DB handle marker and explicit internal row-schema marker.
   - Missing handle or row-schema marker now returns deterministic:
     - status `400`
     - code `DB.QUERY_ONE_INVALID`
     - existing validation message contract (`db.queryOne requires db capability, query, and row schema handles`).
3. Removed compatibility fallback behavior:
   - runtime no longer defaults missing internal DB handle/row-schema markers to `"1"`.
4. Added focused unit coverage in `lasm_db_runtime_dispatch`:
   - `exec_marker_rejects_missing_db_handle_header`
   - `query_one_marker_rejects_missing_row_schema_header`

## Why

Silent defaulting of missing internal DB markers can hide route-planning/header-materialization bugs and let malformed internal operation state execute as if it were valid. Requiring explicit markers keeps DB runtime behavior deterministic and fail-fast.

## Validation

1. `cargo test -p sec4 lasm_db_runtime_dispatch::tests::`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`

