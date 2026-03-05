# M39: LASM Fallback Cache Eligibility Gating

Date: 2026-02-21  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `run_lasm_cluster_accept_loop(...)` in `/compiler/sec4-cli/src/main.rs`.
- Added precomputed `relay_has_next_live_sender_lookup` capability flag.
- Fallback cache wiring now applies only when both are true:
  - lookup storage exists (`relay_has_next_live_sender_lookup`),
  - degraded live-shard count is greater than two.
- Cache refresh path on liveness changes now uses the same eligibility gating.

## Why

After introducing fallback-multi cache usage, option construction and cache-path checks still ran in states where the cache is not useful (`live_count <= 2`). Those states already use specialized single/dual dispatch paths.

## Result

- Reduced unnecessary fallback cache handling in single/dual degraded states.
- Preserved deterministic degraded dispatch behavior and cache-backed fallback behavior in multi-live degraded pools.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
