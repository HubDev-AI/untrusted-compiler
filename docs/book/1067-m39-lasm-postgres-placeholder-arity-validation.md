# 1067 M39 Slice: LASM Postgres Placeholder Arity Validation

This slice adds deterministic SQL placeholder-arity validation for LASM Postgres DB intrinsic execution.

## What changed

1. Added SQL placeholder scanning helper for Postgres query templates to compute max `$N` placeholder index used in a query.
2. Added pre-execution params arity validation:
   - if query template requires placeholder index `N` and provided params length is `< N`, execution fails before sending SQL to Postgres.
3. Applied validation uniformly to LASM Postgres intrinsic paths:
   - `db.exec`
   - `db.execTx`
   - `db.queryOne`
4. Extended command integration coverage with a deterministic missing-param failure case for Postgres `db.queryOne`.

## Why

Without arity validation, under-specified params produce adapter-specific SQL errors.

Deterministic arity diagnostics make runtime failures clear and actionable for generated LASM routes using `sql.q(..., params)`.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
2. `cargo test -p sec4 --test commands run_command_rejects_postgres_adapter_without_dsn`
