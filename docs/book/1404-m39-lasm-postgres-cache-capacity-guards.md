# M39: LASM Postgres Cache Capacity Guards

Date: 2026-02-22  
Milestone: M39 (DB runtime hardening)

## What Changed

- Added bounded cache-size controls in `compiler/sec4-cli/src/lasm_dynamic_state.rs`:
  - `SEC4_RT_LASM_DB_POSTGRES_STATEMENT_CACHE_MAX` (default `512`)
  - `SEC4_RT_LASM_DB_POSTGRES_PLACEHOLDER_CACHE_MAX` (default `1024`)
- Added runtime state fields:
  - `db_postgres_statement_cache_max`
  - `db_postgres_placeholder_cache_max`
- Applied capacity guards:
  - statement cache insert path in `lasm_dynamic_postgres_prepared_statement(...)`
  - placeholder-index cache insert path in `max_lasm_postgres_placeholder_index_cached(...)`
- Cache behavior at capacity is deterministic clear-and-refill.

## Why

Prepared-statement and placeholder caches improved hot-path DB performance, but without bounds their key-space can grow unbounded in long-lived workloads with high template churn. Capacity guards keep caching useful while ensuring bounded memory growth.

## Result

- Postgres runtime caches remain effective for repeated templates.
- Runtime memory growth for those caches is now bounded by explicit limits.
- Existing DB semantics and diagnostics remain unchanged.

## Validation

- `rustfmt compiler/sec4-cli/src/lasm_dynamic_state.rs compiler/sec4-cli/src/lasm_db_runtime_common.rs compiler/sec4-cli/src/lasm_db_runtime_postgres.rs`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
