# M39: LASM Autoscale Recovery Reserve And Spawn Lock-Light

Date: 2026-02-22  
Milestone: M39 (scaling/runtime hot-path tuning)

## What Changed

- Added `reserve_lasm_cluster_min_worker_ports(...)` in lifecycle utilities:
  - reserves missing min-instance worker ports by advancing `next_port` under lock
  - does not spawn processes while holding the cluster-state write lock
- Updated autoscale loop to use reserved recovery ports:
  - recovery worker spawns now run outside the state lock
  - existing autoscale scale-up/scale-down planning still runs under lock
  - worker snapshot refresh still happens after lock re-acquire and state updates

## Why

Even after moving autoscale scale-up/scale-down process lifecycle outside the lock, min-worker recovery still spawned workers while lock-held. That kept a blocking path in the write lock on degraded/recovery cycles.

## Result

- Min-worker recovery avoids lock-held process startup.
- Recovery semantics remain deterministic (`min_instances` guarantee is preserved).
- Autoscale loop keeps lock scope focused on state planning/apply phases.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
