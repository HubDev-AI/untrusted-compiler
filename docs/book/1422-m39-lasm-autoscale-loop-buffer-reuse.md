# M39: LASM Autoscale Loop Buffer Reuse

Date: 2026-02-22  
Milestone: M39 (scaling/runtime hot-path tuning)

## What Changed

- Refactored autoscale loop temporary vectors to be allocated once and reused:
  - `workers_to_spawn_ports`
  - `workers_to_stop`
  - `spawned_workers`
- Switched per-iteration processing to `clear()` + `drain(..)` patterns for those vectors.

## Why

Autoscale loop runs continuously. Re-allocating short-lived vectors every iteration adds avoidable allocation churn in maintenance paths.

## Result

- Same autoscale behavior, lower transient allocation churn in loop internals.
- No semantic change to scale planning or worker lifecycle behavior.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
