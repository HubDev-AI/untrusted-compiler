# 1081 M39 Slice: LASM Postgres Parameterized Single-Statement Precheck

This slice adds deterministic pre-validation for postgres parameterized SQL so multi-statement query shapes are rejected before adapter execution.

## What changed

1. Reused SQL statement-separator scanner to detect non-trailing statement separators while ignoring literals/comments.
2. Added pre-check guard to postgres parameterized runtime paths:
   - `db.exec`
   - `db.execTx`
   - `db.queryOne`
3. Parameterized multi-statement postgres SQL now fails early with deterministic validation diagnostic:
   - `postgres parameterized execution requires a single SQL statement`
4. Added focused postgres command integration assertion for queryOne parameterized multi-statement rejection.

## Why

Previous behavior depended on adapter-native parse errors for parameterized multi-statement SQL.  
Pre-validation makes failure semantics deterministic and aligned with sqlite single-statement guards already added in this milestone.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_sqlite_records_and_query_one_when_db_intrinsics_are_used`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
3. `cargo test -p sec4 --test commands run_command_rejects_postgres_adapter_without_dsn`
