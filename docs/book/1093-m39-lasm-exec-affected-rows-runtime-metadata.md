# 1093 M39 Slice: LASM Exec Affected-Rows Runtime Metadata

This slice adds explicit `affectedRows` metadata to LASM DB write responses for better runtime DB-client observability.

## What changed

1. Updated sqlite runtime execution paths:
   - `run_lasm_sqlite_exec` now returns affected-row count (`u64`)
   - `run_lasm_sqlite_exec_tx` now forwards affected-row count
   - row-producing execution fallback now drains rows and reports drained row count
2. Updated postgres runtime execution paths:
   - `run_lasm_postgres_exec` now returns affected-row count (`u64`)
   - `run_lasm_postgres_exec_tx` now returns affected-row count from transaction execution
   - non-prepared/batch execution paths return deterministic `0` affected rows
3. Wired `affectedRows` into runtime JSON envelopes:
   - `db.exec` response now includes `"affectedRows": <u64>`
   - `db.execTx` response now includes `"affectedRows": <u64>`
   - records-log adapter keeps deterministic `affectedRows: 0`
4. Extended command integration assertions to enforce presence of `affectedRows` in records-log/sqlite/postgres exec flows.

## Why

Before this change, LASM DB write responses exposed only record metadata and operation labels.  
Returning real affected-row counts makes runtime write outcomes more useful to operators and client consumers without changing intrinsic call shapes.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_sqlite_records_and_query_one_when_db_intrinsics_are_used`
3. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
4. `cargo test -p sec4 --test commands run_command_lasm_backend_rejects_stale_exec_tx_handle_after_restart_when_sqlite_adapter`
