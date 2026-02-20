# 1076 M39 Slice: LASM Postgres QueryOne Trailing-Semicolon Normalization

This slice improves LASM Postgres queryOne ergonomics by normalizing terminal semicolons before subquery wrapping.

## What changed

1. Added queryOne normalization helper that trims leading/trailing whitespace and strips trailing `;` suffixes.
2. Applied normalization before:
   - first-keyword SQL-shape checks
   - placeholder-arity validation
   - `row_to_json` wrapper query construction
3. Extended the Postgres integration query fixture to include a trailing semicolon and verified deterministic success.

## Why

Many SQL snippets are copied with terminal semicolons. QueryOne wraps templates into a subquery; unnormalized trailing semicolons can break that wrapper parse.

Normalization keeps valid row-returning queries ergonomic and deterministic.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
2. `cargo test -p sec4 --test commands run_command_rejects_postgres_adapter_without_dsn`
