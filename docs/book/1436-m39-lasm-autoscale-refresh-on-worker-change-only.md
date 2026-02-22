# M39: LASM Autoscale Refresh on Worker Change Only

Date: 2026-02-22  
Milestone: M39 (scaling/runtime hot-path overhead reduction)

## What Changed

- Changed `prune_dead_lasm_cluster_workers` to return a `bool` that reports whether cluster worker membership changed.
- Updated autoscale loop snapshot publishing so the pre-action refresh runs only when worker membership changed in maintenance/scale planning:
  - dead-worker prune changed worker set, or
  - autoscale scale-down popped workers from active set.
- Kept post-action refresh behavior unchanged for spawn/stop apply paths (worker topology changes after process lifecycle operations).

## Why

Autoscale loop still performed a pre-action snapshot refresh on every maintenance tick, even when worker topology was unchanged. That caused avoidable `ArcSwap` snapshot checks and potential clone/publish churn in steady state.

## Result

- Steady-state autoscale ticks no longer refresh worker-port snapshots unless worker membership changed.
- Worker topology updates still publish deterministically on prune/scale-down/spawn/stop changes.
- Relay dispatch behavior remains unchanged; this is maintenance-loop overhead reduction only.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
