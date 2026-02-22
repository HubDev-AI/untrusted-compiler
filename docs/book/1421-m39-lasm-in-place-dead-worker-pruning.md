# M39: LASM In-Place Dead Worker Pruning

Date: 2026-02-22  
Milestone: M39 (scaling/runtime hot-path tuning)

## What Changed

- Replaced dead-worker pruning implementation in cluster lifecycle with in-place removal:
  - old flow drained workers into a temporary vector and rebuilt `state.workers`
  - new flow iterates in place and removes dead/uninspectable entries via `swap_remove`

## Why

The autoscale loop runs maintenance frequently. Rebuilding a temporary worker vector on each prune pass adds avoidable allocation and copy churn in cluster maintenance paths.

## Result

- Pruning now avoids temporary vector rebuilds.
- Existing dead-worker/uninspectable warning messages are preserved.
- Worker liveness semantics remain unchanged.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
