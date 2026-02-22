# M39: LASM Autoscale Process Lifecycle Outside State Lock

Date: 2026-02-22  
Milestone: M39 (scaling/runtime hot-path tuning)

## What Changed

- Refactored LASM cluster autoscale loop to reduce write-lock hold time on cluster state:
  - scale decisions, worker-port reservation, and worker removal planning still happen under the state write lock
  - worker process lifecycle operations (`spawn_and_wait`, `kill`, `wait`) now run outside the lock
  - spawned workers are appended and worker-port snapshots refreshed after re-acquiring the lock

## Why

Spawning or terminating worker processes can block for non-trivial time. Holding the shared cluster-state write lock during these operations increases contention and can delay other cluster maintenance/status paths.

## Result

- Cluster-state lock is held for planning/apply phases, not process lifecycle wait time.
- Autoscale cooldown/target semantics remain deterministic.
- Snapshot publication remains explicit after worker-set changes.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
