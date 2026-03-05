# 1068 M39 Slice: LASM Postgres Literal-Aware Placeholder Rendering

This slice hardens LASM Postgres `$N` placeholder handling to avoid replacing literal placeholder text inside SQL string literals.

## What changed

1. Updated Postgres placeholder scanning to ignore `$N` tokens inside single-quoted SQL string literals (including doubled-quote escapes).
2. Updated Postgres placeholder rendering to replace `$N` placeholders only when outside single-quoted literals.
3. Kept deterministic placeholder-arity validation aligned with the same literal-aware scanning rules.
4. Extended Postgres integration coverage with a route query that includes a literal `'$1-literal'` column and validates literal preservation.

## Why

Naive global `$N` replacement can mutate literal SQL text unexpectedly and break query correctness.

Literal-aware rendering keeps parameter binding deterministic while preserving intended SQL string content.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
2. `cargo test -p sec4 --test commands run_command_rejects_postgres_adapter_without_dsn`
