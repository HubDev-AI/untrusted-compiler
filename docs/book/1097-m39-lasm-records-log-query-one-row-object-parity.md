# 1097 M39 Slice: LASM Records-Log Query-One Row Object Parity

This slice improves LASM `records.log` adapter query-one behavior by returning a structured `rowObject` payload instead of `null`.

## What changed

1. Updated records-log `db.queryOne` fallback materialization in LASM runtime:
   - when a matching persisted record exists, response now includes a non-null `rowObject`
   - `rowObject` contains deterministic metadata fields:
     - `op`
     - `db`
     - `template`
     - `params`
     - `tx`
     - `affected_rows`
     - `rowSchema`
2. Updated string `row` payload for records-log fallback:
   - `row` now serializes the same deterministic metadata object (JSON string), improving shape parity with adapter-backed query payloads.
3. Updated command integration coverage:
   - records-log query-one path now asserts non-null `rowObject`
   - response assertions now include deterministic `affected_rows` presence in fallback query-one payloads.

## Why

Before this change, records-log query-one returned `rowObject: null` while sqlite/postgres returned structured row objects.

Returning structured fallback metadata reduces adapter-specific shape drift for operator/client consumers while keeping records-log semantics deterministic.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_sqlite_records_and_query_one_when_db_intrinsics_are_used`
3. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
