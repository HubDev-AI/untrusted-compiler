# 1075 M39 Slice: LASM Postgres QueryOne Select-Shape Guard

This slice adds deterministic SQL-shape enforcement for LASM Postgres `db.queryOne` execution.

## What changed

1. Added first-keyword extraction helper for SQL templates with whitespace/comment skipping (`--` and nested `/* ... */`).
2. Added `db.queryOne` statement-shape guard before execution:
   - allowed first keywords: `SELECT`, `WITH`, `VALUES`, `TABLE`
   - disallowed statement shapes fail deterministically
3. Extended Postgres integration flow with a non-select `db.queryOne` request (`DELETE ...`) that must fail with deterministic queryOne shape diagnostics.

## Why

`db.queryOne` is a row-returning read path. Running non-select statements there causes ambiguous adapter errors and inconsistent behavior.

Explicit statement-shape guarding keeps runtime behavior deterministic and aligned with intrinsic intent.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
2. `cargo test -p sec4 --test commands run_command_rejects_postgres_adapter_without_dsn`
