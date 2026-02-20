# 1089 M39 Slice: LASM QueryOne Record Persistence Parity

This slice makes LASM runtime DB behavior more consistent by persisting successful sqlite/postgres `db.queryOne` executions as explicit runtime records.

## What changed

1. Updated LASM `queryOne` materialization for sqlite/postgres adapters:
   - successful `db.queryOne` now appends a new DB record with `op = "queryOne"`
   - record IDs now advance deterministically for queryOne success paths
2. Persisted queryOne records through existing adapter persistence flow:
   - sqlite persists to `records.sqlite3`
   - postgres persists to `sec4_lasm_db_records`
3. Updated command integration expectations:
   - sqlite queryOne response now returns queryOne record metadata (`recordId: 3`, `op: "queryOne"`)
   - sqlite/postgres list endpoints now include queryOne records (`count: 3`)

## Why

Before this change, sqlite/postgres queryOne responses reused prior `exec`/`execTx` record metadata and did not append a queryOne runtime record.  
Persisting queryOne operations closes that observability gap and makes adapter-backed DB operation history consistent.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_sqlite_records_and_query_one_when_db_intrinsics_are_used`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
3. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
