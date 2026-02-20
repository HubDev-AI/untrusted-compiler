# 1091 M39 Slice: LASM QueryOne Records-Log Parity

This slice aligns records-log behavior with sqlite/postgres queryOne persistence by appending explicit `queryOne` records on successful records-log fallback matches.

## What changed

1. Updated LASM records-log queryOne fallback path:
   - successful fallback matches now append a new runtime record with `op = "queryOne"`
   - deterministic record IDs now advance on records-log queryOne success, matching sqlite/postgres behavior
2. Persisted those records through the existing records-log persistence flow.
3. Updated records-log command integration expectations:
   - queryOne response now reports `recordId: 3` and `op=queryOne`
   - DB list response now reports `count: 3` and includes `queryOne` operation history

## Why

Before this change, records-log queryOne responses reused prior `exec`/`execTx` record metadata and did not append queryOne history, diverging from sqlite/postgres behavior.

This closes that parity gap and makes queryOne operation history consistent across adapters.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_sqlite_records_and_query_one_when_db_intrinsics_are_used`
