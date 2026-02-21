# M39: LASM Fallback Skip Final Scan Advance

Date: 2026-02-21  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `dispatch_lasm_cluster_relay_stream_fallback_multi(...)` in `/compiler/sec4-cli/src/main.rs`.
- Added scan-budget short-circuit checks in degraded general fallback scan:
  - dead-slot branch now breaks before helper advancement when `scanned_slots + 1 >= scan_slot_limit`,
  - post-attempt branch now applies the same budget guard before helper advancement.

## Why

When scan budget is already exhausted, calling `advance_lasm_cluster_fallback_scan_index(...)` cannot enable another loop iteration. Running helper lookup/scan work on that terminal step adds avoidable hot-path branching.

## Result

- Eliminated terminal advancement helper calls that cannot affect fallback outcomes.
- Preserved bounded scan behavior and deterministic saturated/unavailable envelopes.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
