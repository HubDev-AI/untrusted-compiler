# 1064 M39 Slice: LASM Postgres Client Auto-Reconnect

This slice adds runtime reconnect behavior for the LASM Postgres DB adapter.

## What changed

1. LASM runtime keeps Postgres DSN in dynamic state for reconnect use.
2. Postgres DB intrinsic execution paths now retry once on closed-connection failures:
   - `db.exec`
   - `db.execTx`
   - `db.queryOne`
3. Postgres metadata persistence path now also attempts one reconnect-and-retry when the active client is closed.
4. Reconnect path re-initializes schema checks before retrying execution.

## Why

Persistent clients reduce connection overhead, but long-running processes can still hit transient connection drops.

Auto-reconnect keeps LASM workers resilient without requiring process restarts, which is required for stable alpha behavior under sustained load.

## Validation

1. `cargo test -p sec4 --test commands run_command_rejects_postgres_adapter_without_dsn`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
3. Regression checks:
   - `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
   - `cargo test -p sec4 --test commands run_command_lasm_backend_persists_sqlite_records_and_query_one_when_db_intrinsics_are_used`
