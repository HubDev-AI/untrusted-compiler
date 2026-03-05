# M39: LASM Fallback Scan Helper Direct Lookup Path

Date: 2026-02-21  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `advance_lasm_cluster_fallback_scan_index(...)` in `/compiler/sec4-cli/src/main.rs`.
- After dead wrapped-next detection, helper now uses:
  - direct lookup path (`lookup_lasm_cluster_next_live_sender_index`) when lookup slice is available,
  - direct scan path (`lasm_cluster_next_live_sender_index`) when lookup slice is empty.
- Removed round-trip through `resolve_lasm_cluster_next_live_sender_index(...)` inside helper advancement path.

## Why

The helper already performs wrapped-next live fast-path checks and then routed into a more generic resolver, introducing extra branch plumbing in the helper hot path.

## Result

- Simpler branch path in scan helper advancement.
- Preserved deterministic next-live resolution semantics for both cached and non-cached states.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
