# M39: Postgres Cache Eviction Telemetry

Date: 2026-02-22  
Milestone: M39 (DB runtime observability)

## What Changed

- Added cumulative Postgres cache-eviction counters in LASM dynamic runtime state:
  - `db_postgres_statement_cache_evictions_total`
  - `db_postgres_placeholder_cache_evictions_total`
- Wired counter increments on capacity-triggered cache clears for:
  - prepared statement cache
  - placeholder-index cache
- Extended `DbListRecordsResponse` `dbCache` telemetry with:
  - `postgresStatementEvictedTotal`
  - `postgresPlaceholderEvictedTotal`
- Updated command integration assertions for deterministic presence of new telemetry fields.

## Why

Count/capacity snapshots alone do not show cache churn. Operators need cumulative eviction signals to detect undersized cache settings under live workloads.

## Result

- `/db/records` now exposes cache-churn telemetry for Postgres runtime tuning.
- Placeholder cache eviction accounting has direct unit coverage.

## Validation

- `rustfmt compiler/sec4-cli/src/lasm_dynamic_state.rs compiler/sec4-cli/src/lasm_db_runtime_common.rs compiler/sec4-cli/src/lasm_db_runtime_postgres.rs compiler/sec4-cli/src/main.rs compiler/sec4-cli/tests/commands.rs`
- `cargo test -p sec4 placeholder_cache_eviction_counter_increments_when_capacity_is_hit`
- `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
