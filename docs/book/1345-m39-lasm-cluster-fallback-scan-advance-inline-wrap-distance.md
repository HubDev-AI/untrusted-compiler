# M39: LASM Cluster Fallback Scan Advance Inline Wrap Distance

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `$REPO_ROOT/compiler/sec4-cli/src/lasm_cluster_fallback_dispatch.rs`:
  - `advance_lasm_cluster_fallback_scan_index` now computes:
    - wrapped next start index,
    - forward-distance slot advancement,
    directly in-function.
  - removed separate wrapped-forward-distance helper indirection.

## Why

This helper executes in fallback scan loops. Keeping wrapped-step and slot-distance arithmetic local removes one helper layer in this hot path while maintaining identical scan progression.

## Result

- Reduced helper indirection in fallback scan-advance path.
- No behavior change in scan cursor progression or slot accounting.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
