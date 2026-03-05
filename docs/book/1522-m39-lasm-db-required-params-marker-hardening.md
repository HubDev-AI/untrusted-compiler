# 1522 M39 Slice: LASM DB Required Params Marker Hardening

This slice hardens LASM internal DB dispatch by removing silent fallback defaults for missing internal SQL params markers.

## What changed

1. Hardened `db.exec` marker handling:
   - runtime now requires explicit internal SQL params marker,
   - missing params marker now returns deterministic:
     - status `400`
     - code `DB.EXEC_INVALID`.
2. Hardened `db.execTx` marker handling:
   - runtime now requires explicit internal SQL params marker,
   - missing params marker now returns deterministic:
     - status `400`
     - code `DB.EXEC_TX_INVALID`.
3. Hardened `db.queryOne` marker handling:
   - runtime now requires explicit internal SQL params marker,
   - missing params marker now returns deterministic:
     - status `400`
     - code `DB.QUERY_ONE_INVALID`.
4. Removed compatibility fallback behavior:
   - runtime no longer defaults missing internal SQL params markers to `"0"` in DB materialization paths.
5. Added focused unit coverage:
   - `exec_marker_rejects_missing_params_header`
   - `query_one_marker_rejects_missing_params_header`

## Why

Silent defaulting of missing SQL params markers can hide internal route-plan/header-materialization bugs and produce misleading runtime behavior. Requiring explicit markers keeps DB intrinsic execution deterministic and fail-fast.

## Validation

1. `cargo test -p sec4 lasm_db_runtime_dispatch::tests::`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`

