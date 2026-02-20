# 1100 M39 Slice: LASM DB Tx-Handle Capacity Guard

This slice adds a bounded tx-handle registry for LASM DB runtime execution.

## What changed

1. Added tx-handle capacity configuration for LASM dynamic DB state:
   - env var: `SEC4_RT_LASM_DB_MAX_TX_HANDLES`
   - default: `256`
   - invalid values (`0`, non-numeric) now fail deterministically during run startup
2. Updated tx-handle allocation behavior:
   - `allocate_lasm_db_tx_handle(...)` now returns `None` when the registry reaches configured capacity
3. Updated `db.execTx` inline `db.tx(...)` path:
   - when tx-handle capacity is exhausted, runtime now returns deterministic envelope:
     - status: `500`
     - code: `DB.TX_INTERNAL`
     - message: `db.tx runtime failure`
   - no additional DB record is appended on capacity failure
4. Added focused integration coverage:
   - single-process sqlite LASM run with `SEC4_RT_LASM_DB_MAX_TX_HANDLES=1`
   - first `db.execTx` request succeeds
   - second request fails with `DB.TX_INTERNAL`
   - records list remains at one execTx record (no overflow side effects)

## Why

Before this change, LASM tx-handle registry growth was unbounded for long-lived server processes.

Bounding tx-handle capacity aligns behavior with deterministic runtime guardrails and prevents silent handle-map growth under sustained execTx traffic.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_enforces_db_tx_handle_capacity_when_sqlite_adapter`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_sqlite_records_and_query_one_when_db_intrinsics_are_used`
3. `cargo test -p sec4 --test commands run_command_lasm_backend_invalid_exec_tx_handle_does_not_execute_sql_when_sqlite_adapter`
4. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
