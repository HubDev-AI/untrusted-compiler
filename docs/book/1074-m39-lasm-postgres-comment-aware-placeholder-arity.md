# 1074 M39 Slice: LASM Postgres Comment-Aware Placeholder Arity

This slice hardens LASM Postgres placeholder-arity validation by ignoring placeholder-like tokens in SQL comments.

## What changed

1. Extended `max_lasm_postgres_placeholder_index(...)` scanner to treat SQL comments as non-placeholder regions:
   - line comments: `-- ...`
   - block comments: `/* ... */`
   - nested block comments supported by depth tracking
2. Kept existing placeholder scanning protections for single-quoted and dollar-quoted literals.
3. Updated Postgres integration query fixture to include a block comment containing `$9` and verified normal execution with only three provided params.

## Why

Without comment awareness, `$N` tokens inside comments can cause false arity failures (`requires at least N params`) even when executable placeholders are valid.

Comment-aware scanning keeps deterministic validation strict for real placeholders while ignoring non-executable comment text.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
2. `cargo test -p sec4 --test commands run_command_rejects_postgres_adapter_without_dsn`
