# M39: LASM Cluster Relay Shared Selection Recompute and Fallback Lookup Traversal

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `$REPO_ROOT/compiler/sec4-cli/src/lasm_cluster_relay_worker_loop.rs`:
  - added shared helper `recompute_lasm_cluster_relay_selection_state(...)` for selection-state derivation (`has_healthy`, identity mode, cycle span, single-healthy index),
  - steady-path selection rebuild now calls this helper,
  - connect-failure fallback now refreshes selection state once and traverses fallback candidates from healthy lookup state instead of scanning all worker slots.

## Why

Selection-state logic existed in multiple branches and fallback traversal still touched unhealthy slots. One recompute helper plus healthy-lookup traversal keeps failure-path work tighter and avoids drift between steady and failure selection semantics.

## Result

- Shared deterministic selection-state derivation across relay branches.
- Fallback connect attempts iterate healthy candidates directly.
- No change to response envelopes, warning throttling, or final saturation semantics.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
