# M39: LASM Fallback-Multi Next-Live Cache Usage

Date: 2026-02-21  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `/compiler/sec4-cli/src/main.rs`:
  - added `resolve_lasm_cluster_next_live_sender_index(...)`,
  - extended `dispatch_lasm_cluster_relay_stream_fallback_multi(...)` with optional next-live lookup input,
  - wired accept-loop fallback calls to pass cached lookup slice when available.
- Degraded fallback branches now resolve live indices through cached lookup first in `scan_live_target` 0/1/2 paths, with deterministic scan fallback when cache entries are stale/missing.

## Why

Fallback-multi still performed repeated live-index scans even after introducing accept-loop next-live cache support. Reusing the same cache in fallback selection reduces additional scan work in degraded fallback-heavy paths.

## Result

- Lower degraded fallback scan overhead in multi-shard pools.
- Preserved deterministic saturated/unavailable behavior and disconnect handling semantics.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
