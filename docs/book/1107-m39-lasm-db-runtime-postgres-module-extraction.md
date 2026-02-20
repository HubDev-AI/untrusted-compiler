# 1107 M39 Slice: LASM DB Runtime Postgres Module Extraction

This slice continues adapter-layer extraction by moving PostgreSQL runtime DB execution helpers out of `main.rs`.

## What changed

1. Added module `compiler/sec4-cli/src/lasm_db_runtime_postgres.rs`.
2. Moved PostgreSQL runtime helpers from `main.rs`:
   - `LasmPostgresParam` + `parse_lasm_postgres_query_params`
   - placeholder/dollar-quote parsing and placeholder max-index analysis
   - select-shape detection + query normalization for queryOne wrappers
   - `run_lasm_postgres_exec`
   - `run_lasm_postgres_exec_tx`
   - `run_lasm_postgres_query_one`
3. Kept call sites in `main.rs` using imported module helpers.
4. Kept reconnect + single-statement guard behavior unchanged.

## Why

Postgres runtime execution logic is adapter-specific behavior and should be isolated from CLI/request orchestration in `main.rs`.

This extraction materially shrinks `main.rs` and establishes a dedicated runtime-exec boundary for postgres adapter internals.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_sqlite_records_and_query_one_when_db_intrinsics_are_used`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
