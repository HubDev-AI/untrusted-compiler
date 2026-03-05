# 1099 M39 Slice: LASM DB List-Records Affected-Rows Aggregate

This slice extends `DbListRecordsResponse` with a deterministic aggregate write-impact field.

## What changed

1. Added aggregate computation in LASM response materialization for `DbListRecordsResponse`:
   - computes `affectedRowsTotal` as saturating sum of `record.affected_rows` across persisted records
2. Extended response payload shape:
   - list response now includes:
     - `count`
     - `affectedRowsTotal`
     - `adapter`
     - `records`
3. Updated command integration assertions for list responses across adapters:
   - records-log list response now checks `affectedRowsTotal`
   - sqlite list response now checks `affectedRowsTotal`
   - postgres list response now checks `affectedRowsTotal`

## Why

`affected_rows` per record is useful, but operators also need a fast cumulative signal from `DbListRecordsResponse` without post-processing each record.

Adding `affectedRowsTotal` keeps contracts deterministic and improves DB runtime observability.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_sqlite_records_and_query_one_when_db_intrinsics_are_used`
3. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
