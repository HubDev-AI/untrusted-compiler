# 1070 M39 Slice: LASM Postgres Dollar-Quoted Placeholder Preservation

This slice extends LASM Postgres placeholder handling to preserve `$N` text inside Postgres dollar-quoted string literals.

## What changed

1. Added Postgres dollar-quote delimiter parsing (`$$...$$` and `$tag$...$tag$`) for query-template scanning.
2. Updated placeholder-arity scanning to skip `$N` tokens when inside active dollar-quoted literals.
3. Updated placeholder rendering to skip replacement inside dollar-quoted literals while still replacing bind placeholders outside literal sections.
4. Extended Postgres command integration coverage with a `dollar_literal` column (`$$$2-dollar$$`) to prove literal preservation alongside normal parameter binding.

## Why

Postgres queries often embed literal SQL text using dollar-quoted blocks. Placeholder replacement must not alter those literal payloads.

Dollar-quote-aware scanning/rendering keeps parameter binding deterministic and prevents accidental SQL text mutation.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
2. `cargo test -p sec4 --test commands run_command_rejects_postgres_adapter_without_dsn`
