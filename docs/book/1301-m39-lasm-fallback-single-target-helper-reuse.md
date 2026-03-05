# M39: LASM Fallback Single-Target Helper Reuse

Date: 2026-02-21  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `dispatch_lasm_cluster_relay_stream_fallback_multi(...)` in `/compiler/sec4-cli/src/main.rs`.
- In the `scan_live_target <= 1` branch:
  - replaced inline resolved-target send/disconnect logic with direct delegation to `dispatch_lasm_cluster_relay_stream_fallback_single_live(...)`.

## Why

Single-target fallback handling in this branch duplicated the same behavior already implemented in the dedicated single-live helper. Reusing the helper keeps one deterministic implementation for this path.

## Result

- Reduced duplicate send/disconnect code in fallback single-target branch.
- Preserved deterministic saturated/unavailable outcomes and liveness updates.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
