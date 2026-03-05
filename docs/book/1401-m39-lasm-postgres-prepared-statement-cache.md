# M39: LASM Postgres Prepared Statement Cache

Date: 2026-02-22  
Milestone: M39 (DB runtime hardening)

## What Changed

- Added a Postgres prepared-statement cache to `LasmDynamicResponseState`:
  - `db_records_postgres_statement_cache: HashMap<String, postgres::Statement>`
- Added shared runtime helper:
  - `lasm_dynamic_postgres_prepared_statement(...)` in `compiler/sec4-cli/src/lasm_db_runtime_common.rs`
- Wired Postgres runtime operations to cached statements:
  - `run_lasm_postgres_exec`
  - `run_lasm_postgres_exec_tx`
  - `run_lasm_postgres_query_one` (including wrapped query form)
- Reconnect flow now clears statement cache when rebuilding Postgres client:
  - `reconnect_lasm_dynamic_postgres_client(...)`

## Why

Without caching, repeated Postgres runtime calls re-prepared SQL text on each execution path. That adds avoidable parse/prepare overhead under sustained load. Caching prepared statements by query template keeps execution semantics unchanged while reducing repeated setup work on hot DB paths.

## Result

- Repeated Postgres query templates now reuse prepared statements across requests.
- Reconnect/retry paths remain deterministic and safe: cache is cleared on reconnect, then statements are prepared again against the new client.
- No language-semantic changes were introduced; this is runtime execution-path optimization only.

## Validation

- `rustfmt compiler/sec4-cli/src/lasm_dynamic_state.rs compiler/sec4-cli/src/lasm_db_runtime_common.rs compiler/sec4-cli/src/lasm_db_runtime_postgres.rs`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
