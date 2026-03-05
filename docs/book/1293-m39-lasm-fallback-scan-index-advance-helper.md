# M39: LASM Fallback Scan-Index Advance Helper

Date: 2026-02-21  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Added `advance_lasm_cluster_fallback_scan_index(...)` in `/compiler/sec4-cli/src/main.rs`.
- Replaced duplicated jump progression logic in fallback-multi general degraded scan loop with helper calls in:
  - dead-slot branch,
  - post-attempt progression branch.

## Why

The same jump/index-resolution and wrapped-distance accounting logic was duplicated in two hot branches, increasing maintenance risk and branch drift probability.

## Result

- One shared path for scan-index jump progression.
- Preserved existing next-slot-live fast path, next-live resolver behavior, and wrapped distance accounting semantics.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
