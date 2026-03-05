# M39: LASM Cluster Fallback Live-Target Direct Guards

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `$REPO_ROOT/compiler/sec4-cli/src/lasm_cluster_fallback_dispatch.rs`:
  - fallback scan slot limit now uses direct `sender_count - 1` under existing `sender_count > 1` invariant,
  - fallback live-target derivation now uses direct guarded decrement (`live_count - 1`) instead of `saturating_sub` for:
    - initial `scan_live_target`,
    - dynamic `scan_live_target_dynamic` refresh after live-count changes.

## Why

This path already has explicit invariants for sender/live counts. Direct guarded arithmetic removes redundant saturating operations from fallback scan setup and live-target updates in a hot dispatch path.

## Result

- Reduced arithmetic overhead in fallback scan setup/update path.
- No behavior change in fallback sender-scan bounds.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
