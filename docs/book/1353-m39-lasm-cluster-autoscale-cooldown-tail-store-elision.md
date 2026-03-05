# M39: LASM Cluster Autoscale Cooldown Tail Store Elision

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `$REPO_ROOT/compiler/sec4-cli/src/lasm_cluster_autoscale_loop.rs`:
  - added `cooldown_anchor_changed` tracking for per-tick scale actions,
  - end-of-tick cooldown remaining atomic stores now run only when `last_scale_up_at` or `last_scale_down_at` changed in that tick.

## Why

Cooldown remaining atomics are already written once at the start of each autoscale evaluation tick. When no scale action occurred, the second write at loop tail repeated the same value.

## Result

- Removed duplicate cooldown atomic writes on steady-state no-op ticks.
- Preserved immediate cooldown publication semantics after successful scale-up/down actions.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
