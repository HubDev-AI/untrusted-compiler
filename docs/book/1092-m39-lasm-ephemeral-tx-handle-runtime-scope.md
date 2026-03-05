# 1092 M39 Slice: LASM Ephemeral Tx-Handle Runtime Scope

This slice hardens LASM transaction-handle semantics by keeping `db.tx` handles runtime-local only.

## What changed

1. Updated LASM dynamic state bootstrap:
   - removed tx-handle rehydration from persisted DB record history on startup
   - tx-handle map now starts empty for each runtime process
   - next tx-handle counter now starts at `1` per runtime process
2. Added a focused restart regression test:
   - run 1 seeds a valid `db.execTx(db.tx(...), ...)` record
   - run 2 attempts to reuse stale handle `tx=1` through internal execTx headers
   - runtime now deterministically returns `400 DB.EXEC_TX_HANDLE_INVALID`
   - records remain unchanged (no stale-handle SQL side effects)

## Why

Persisted DB records are history, not live transaction capability state.

Rehydrating tx handles from persisted records could allow stale handle reuse across restarts, violating the `db.tx` capability contract.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_rejects_stale_exec_tx_handle_after_restart_when_sqlite_adapter`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_invalid_exec_tx_handle_does_not_execute_sql_when_sqlite_adapter`
3. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_sqlite_records_and_query_one_when_db_intrinsics_are_used`
