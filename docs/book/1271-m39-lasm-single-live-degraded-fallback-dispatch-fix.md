# M39: LASM Single-Live Degraded Fallback Dispatch Fix

Date: 2026-02-21  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `dispatch_lasm_cluster_relay_stream_fallback_multi` in `/compiler/sec4-cli/src/main.rs`.
- Added an explicit `scan_live_target == 0` branch that:
  - resolves the remaining live sender from `relay_sender_live`,
  - attempts one `try_send` to that shard,
  - preserves deterministic saturated/unavailable mapping after the attempt.

## Why

When relay pools had more than two sender shards and degraded to exactly one live shard, fallback logic could return unavailable without attempting the remaining live shard. This produced avoidable request failures in degraded states.

## Result

- Single-live degraded fallback now attempts the last live shard before failing.
- Deterministic error behavior is preserved (`Saturated` if a live shard was observed, otherwise `Unavailable`).
- Keeps existing multi-shard and dual-shard fallback semantics unchanged.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
