# 1098 M39 Slice: LASM DB Capability Handle Validation Guard

This slice hardens LASM DB intrinsic runtime behavior by rejecting invalid DB capability handles before adapter execution.

## What changed

1. Added DB capability handle validation helper in LASM runtime path:
   - accepted DB capability handle is now deterministic (`db == 1`)
2. Applied guard to `db.exec` materialization:
   - invalid DB handles now return deterministic `400 DB.EXEC_INVALID`
   - adapter execution is skipped for invalid handles
3. Applied guard to `db.queryOne` materialization:
   - invalid DB handles now return deterministic `400 DB.QUERY_ONE_INVALID`
   - adapter execution is skipped for invalid handles
4. Applied guard to inline `db.tx(...)` source path used by `db.execTx`:
   - invalid DB handles in `db.execTx(db.tx(invalidDb), query)` now return deterministic `400 DB.EXEC_TX_INVALID`
   - tx-handle allocation and adapter execution are skipped
5. Added command regression coverage:
   - new sqlite-focused integration test proves invalid DB-handle `db.exec` internal path fails and produces no persistent side effects (`count=0` in `DbListRecordsResponse`).

## Why

Before this slice, LASM runtime accepted any positive DB handle value and could execute SQL for manually injected non-capability handles.

Validating DB capability handles at runtime closes that gap and aligns behavior with typed-capability expectations without changing intrinsic signatures.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_invalid_exec_db_handle_does_not_execute_sql_when_sqlite_adapter`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_invalid_exec_tx_handle_does_not_execute_sql_when_sqlite_adapter`
3. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_sqlite_records_and_query_one_when_db_intrinsics_are_used`
4. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
