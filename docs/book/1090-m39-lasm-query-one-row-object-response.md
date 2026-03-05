# 1090 M39 Slice: LASM QueryOne Structured Row Object Response

This slice upgrades LASM DB-client response ergonomics by exposing a structured JSON `rowObject` in `db.queryOne` runtime responses for sqlite/postgres adapters.

## What changed

1. Extended LASM `db.queryOne` success envelopes:
   - sqlite/postgres responses now include `rowObject` (typed JSON object)
   - existing `row` (serialized JSON string) is preserved for compatibility
2. Kept adapter compatibility behavior explicit:
   - records-log fallback keeps deterministic `rowObject: null`
3. Updated command integration coverage:
   - records-log test now asserts deterministic `rowObject: null`
   - sqlite test now asserts deterministic `rowObject` payload
   - postgres test now asserts typed fields via `rowObject`

## Why

The previous `row` string required a second JSON parse by runtime consumers.  
Adding `rowObject` gives immediate structured access for DB-client flows while preserving existing response contracts.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_sqlite_records_and_query_one_when_db_intrinsics_are_used`
3. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
