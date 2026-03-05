# 1521 M39 Slice: LASM DB `execTx` Dual Tx-Source Marker Rejection

This slice hardens LASM internal DB dispatch for `db.execTx` by rejecting ambiguous transaction-source marker sets.

## What changed

1. Hardened `db.execTx` marker validation in `apply_lasm_internal_db_operation_materialization_single`:
   - runtime now checks internal marker shape before tx-handle/db-source resolution,
   - when both transaction-source markers are present at once:
     - internal tx handle marker
     - internal `db.tx(dbCap)` source marker
   - runtime now returns deterministic validation failure:
     - status `400`
     - code `DB.EXEC_TX_INVALID`
     - message: `db.execTx must include either tx handle or db.tx(dbCap) source, not both`.
2. Added focused unit coverage:
   - `exec_tx_marker_rejects_ambiguous_dual_transaction_sources`.

## Why

Dual-source marker sets are planner/header-materialization bugs and should fail fast. Previously runtime silently preferred one source path, which could hide invalid internal state and make failures harder to diagnose.

## Validation

1. `cargo test -p sec4 lasm_db_runtime_dispatch::tests::`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`

