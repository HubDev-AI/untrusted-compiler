# 1055 M39 Slice: LASM Cluster Snapshot Publish Change Detection

This slice reduces background autoscale-loop overhead by publishing worker-port snapshots only when the effective worker-port set changes.

## What changed

1. Replaced unconditional snapshot refresh helper with change-aware refresh:
   - `refresh_lasm_cluster_worker_ports_snapshot_if_changed(...)`
2. Autoscale loop now tracks last published worker-port set and skips snapshot store when:
   - worker count is unchanged,
   - worker port ordering/content is unchanged.
3. Snapshot store now occurs only on real worker-set transitions (prune/recover/scale mutations).

## Why

With lock-free relay snapshots in place, the remaining overhead moved to background snapshot publication. Unconditional republish on every maintenance tick creates avoidable allocations and atomic-store churn during steady state.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`

## Notes

- This is an incremental optimization of the autoscale maintenance loop.
- It preserves existing relay selection behavior and snapshot semantics.
