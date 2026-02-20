# 1079 M39 Slice: LASM DB Runtime Validation Error Classification

This slice upgrades LASM DB runtime error envelopes so known query-validation failures are reported as deterministic `400 ..._INVALID` responses instead of generic `500 ..._FAILED`.

## What changed

1. Added shared LASM DB runtime error classifier used by sqlite/postgres intrinsic execution paths.
2. Preserved deterministic adapter-config classification (`DB.ADAPTER_CONFIG_INVALID`, `500`, `internal`) for missing DSN / adapter bootstrap configuration issues.
3. Classified known validation failures as deterministic `400` validation envelopes:
   - sql-parameter lower-bound failures (`requires at least`)
   - queryOne SQL-shape guard failures (`SELECT`-style requirement)
   - queryOne non-empty SQL requirement
   - parameterized multi-statement constraint failures
4. Applied classifier to runtime DB failure paths for:
   - `db.exec`
   - `db.execTx`
   - `db.queryOne`
5. Updated sqlite/postgres integration assertions to lock `400 ..._INVALID` semantics for queryOne validation failures.

## Why

Validation errors are user-input/query-shape issues, not adapter infrastructure failures.  
Returning deterministic `400` envelopes improves operator feedback and keeps DB runtime contract behavior explicit and stable.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_sqlite_records_and_query_one_when_db_intrinsics_are_used`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
3. `cargo test -p sec4 --test commands run_command_rejects_postgres_adapter_without_dsn`
