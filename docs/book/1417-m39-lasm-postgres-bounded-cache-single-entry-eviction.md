# M39: LASM Postgres Bounded Cache Single-Entry Eviction

Date: 2026-02-22  
Milestone: M39 (runtime/DB performance tuning)

## What Changed

- Replaced clear-all cache behavior in LASM Postgres runtime with bounded single-entry eviction.
- Applied to both:
  - prepared statement cache
  - placeholder-max cache
- Added deterministic cache-order tracking in LASM dynamic state to evict oldest entries when cache capacity is reached.
- Reconnect flow now also resets statement-cache order state together with the statement cache map.

## Why

The previous behavior cleared the full cache map once capacity was hit. Under mixed query-template workloads this causes repeated prepare/scan churn spikes and avoidable latency jitter.

## Result

- Cache capacity is still deterministic and bounded.
- Eviction counters remain deterministic and now increment per evicted entry instead of full-cache resets.
- Runtime avoids full cache flushes on normal capacity pressure.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
- `cargo test -p sec4 lasm_db_runtime_common::tests::bounded_cache_entry_evicts_oldest_single_entry_when_full`
