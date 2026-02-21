# M39: LASM Fallback Single-Attempt Branch Merge

Date: 2026-02-21  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `dispatch_lasm_cluster_relay_stream_fallback_multi(...)` in `/compiler/sec4-cli/src/main.rs`.
- Collapsed duplicated fallback single-attempt branches:
  - merged `scan_live_target == 0` and `scan_live_target == 1` into one `scan_live_target <= 1` path,
  - kept the same single live-index resolve + send/disconnect handling.

## Why

The two branches contained identical logic and only differed in the branch condition. Keeping both adds unnecessary hot-path branch depth and duplicate maintenance surface.

## Result

- Reduced fallback branch shape in degraded single-attempt paths.
- Preserved deterministic saturated/unavailable behavior and liveness transitions.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
