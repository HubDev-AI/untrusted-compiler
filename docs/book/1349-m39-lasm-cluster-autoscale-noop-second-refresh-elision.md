# M39: LASM Cluster Autoscale No-Op Second Refresh Elision

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `$REPO_ROOT/compiler/sec4-cli/src/lasm_cluster_autoscale_loop.rs`:
  - autoscale loop now tracks whether workers changed after the initial maintenance refresh,
  - second worker-port snapshot refresh now runs only when scale-up/scale-down actually changed worker membership.

## Why

The loop already publishes worker-port snapshot once per tick before autoscale decisions. When no scaling happened, running the second refresh only repeated worker-port equality checks with no state change.

## Result

- Reduced redundant worker-port snapshot comparison/publish work in steady-state autoscale ticks.
- Preserved snapshot correctness when workers are actually added/removed.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
