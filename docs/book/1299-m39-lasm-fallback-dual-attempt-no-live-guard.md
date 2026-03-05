# M39: LASM Fallback Dual-Attempt No-Live Guard

Date: 2026-02-21  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `dispatch_lasm_cluster_relay_stream_fallback_multi(...)` in `/compiler/sec4-cli/src/main.rs`.
- In the `scan_live_target == 2` branch, added an immediate guard after first-attempt dispatch:
  - if disconnect handling reduces `relay_live_sender_count` to `0`, fallback now returns immediately,
  - second-live resolver/lookup branch is skipped in that terminal state.

## Why

When live relay count reaches zero after the first dual-attempt branch send, second-live resolution cannot succeed. Continuing into lookup/resolver work adds avoidable hot-path branch overhead.

## Result

- Immediate terminal exit for dual-attempt no-live transitions.
- Preserved deterministic saturated/unavailable fallback semantics.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
