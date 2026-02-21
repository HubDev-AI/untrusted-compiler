# M39: LASM Fallback Jump-Distance Accounting

Date: 2026-02-21  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Added `lasm_cluster_forward_distance_wrapped(...)` in `/compiler/sec4-cli/src/main.rs`.
- Updated `dispatch_lasm_cluster_relay_stream_fallback_multi(...)` general degraded scan loop so `scanned_slots` advances by wrapped jump distance (not always `+1`) in:
  - dead-slot jump path,
  - post-attempt jump-to-next-candidate path.

## Why

After jump-ahead cursor movement, `scanned_slots += 1` undercounted traversal progress and kept loop iteration counts higher than needed in sparse-live degraded pools.

## Result

- Scan-loop bounded progression now matches actual wrapped cursor movement.
- Fewer loop iterations in degraded jump-heavy scans while preserving deterministic traversal bounds and fallback outcomes.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
