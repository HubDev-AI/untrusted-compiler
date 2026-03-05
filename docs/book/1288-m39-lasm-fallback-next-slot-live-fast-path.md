# M39: LASM Fallback Next-Slot-Live Fast Path

Date: 2026-02-21  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `dispatch_lasm_cluster_relay_stream_fallback_multi(...)` in `/compiler/sec4-cli/src/main.rs`.
- In both jump-advance branches (dead-slot path and post-attempt path), fallback loop now:
  - checks whether wrapped `next_scan_start` slot is live,
  - uses it directly when live,
  - calls next-live resolver only when wrapped next slot is dead.

## Why

Resolver dispatch (cache + fallback scan) is unnecessary when the immediate wrapped next slot is already live. This adds avoidable branch/lookup work in steady degraded fallback paths.

## Result

- Lower per-iteration overhead in degraded fallback jump paths.
- Preserved deterministic scan progression and fallback outcomes.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
