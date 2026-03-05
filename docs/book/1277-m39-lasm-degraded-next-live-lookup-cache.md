# M39: LASM Degraded Next-Live Lookup Cache

Date: 2026-02-21  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Added two helpers in `/compiler/sec4-cli/src/main.rs`:
  - `refresh_lasm_cluster_next_live_sender_lookup(...)`
  - `lookup_lasm_cluster_next_live_sender_index(...)`
- `run_lasm_cluster_accept_loop(...)` now keeps a precomputed per-slot `next live relay index` lookup vector for multi-sender pools.
- Degraded-mode dispatch now uses the lookup cache for:
  - cursor recovery when current cursor points to a dead shard (`live_count > 2`),
  - next dispatch index selection when wrapped next slot is dead.
- Lookup cache refresh is triggered only when relay liveness changes (disconnect-driven live-count delta path).

## Why

Degraded relay pools with dead slot gaps caused repeated per-request live-index scan lookups in accept-loop cursor/next-index selection. Under sustained load, those repeated scans add avoidable hot-path overhead.

## Result

- Degraded dispatch keeps deterministic behavior but reduces repeated live-scan work in steady-state degraded pools.
- Liveness-change paths retain explicit refresh boundaries, so cached lookup safety stays tied to known disconnect transitions.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
