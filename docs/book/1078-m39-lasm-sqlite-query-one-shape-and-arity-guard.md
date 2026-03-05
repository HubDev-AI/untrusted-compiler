# 1078 M39 Slice: LASM SQLite QueryOne Shape and Arity Guard

This slice tightens sqlite query execution contracts for LASM DB intrinsics with deterministic validation and queryOne SQL-shape guards.

## What changed

1. Added sqlite sql-parameter arity validation helper used by runtime sqlite execution paths.
2. Enforced deterministic lower-bound parameter checks before sqlite execution/query:
   - if required placeholders exceed provided params, return a deterministic runtime error.
3. Added sqlite queryOne SQL-shape guard:
   - only `SELECT`/`WITH`/`VALUES`/`TABLE` statements are allowed.
4. Added sqlite queryOne trailing-semicolon normalization before SQL-shape check + statement prepare.
5. Extended existing sqlite command integration flow with deterministic negative assertions for:
   - queryOne parameter mismatch
   - queryOne non-row-returning SQL

## Why

Without explicit sqlite guards, invalid queryOne/sql-parameter flows depended on adapter-native error text and could be less predictable.

This keeps LASM sqlite DB-client behavior deterministic and aligned with the stricter Postgres queryOne path.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_sqlite_records_and_query_one_when_db_intrinsics_are_used`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
3. `cargo test -p sec4 --test commands run_command_rejects_postgres_adapter_without_dsn`
