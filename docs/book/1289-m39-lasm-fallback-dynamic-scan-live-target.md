# M39: LASM Fallback Dynamic Scan Live Target

Date: 2026-02-21  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `dispatch_lasm_cluster_relay_stream_fallback_multi(...)` in `/compiler/sec4-cli/src/main.rs`.
- In the general degraded scan branch (`scan_live_target > 2`), loop now uses dynamic scan target state:
  - `scan_live_target_dynamic` initialized from pre-loop live target,
  - updated when disconnect events reduce `relay_live_sender_count`.

## Why

General degraded fallback scan previously compared against a fixed pre-loop live target. If fallback encountered disconnects and live-count dropped, loop bounds could remain stale and continue scanning more than needed.

## Result

- Loop exits according to current live-count after disconnects.
- Preserves bounded fallback traversal and deterministic saturation/unavailable behavior.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
