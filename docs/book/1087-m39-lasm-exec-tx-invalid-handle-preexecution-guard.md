# 1087 M39 Slice: LASM `execTx` Invalid-Handle Pre-Execution Guard

This slice fixes runtime `db.execTx` ordering so invalid existing tx-handle requests are rejected before adapter SQL execution.

## What changed

1. Reordered `db.execTx` runtime handling in LASM materialization:
   - for existing tx-handle paths, validate `tx -> db` binding first
   - only execute adapter SQL after validation succeeds
2. Preserved existing behavior for allocate-from-db tx paths (`db.tx(db)` source).
3. Added a focused sqlite command integration regression:
   - injects internal `execTx` operation with invalid tx handle
   - query template attempts sqlite side-effect insert
   - verifies deterministic `400 DB.EXEC_TX_HANDLE_INVALID`
   - verifies persisted record list remains unchanged (`count: 0`)

## Why

Before this fix, invalid existing tx-handle paths could still execute adapter SQL before failing handle validation, creating side-effect risk.

The new ordering enforces validation-first semantics and prevents side effects on invalid tx handles.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_invalid_exec_tx_handle_does_not_execute_sql_when_sqlite_adapter`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_sqlite_records_and_query_one_when_db_intrinsics_are_used`
3. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
