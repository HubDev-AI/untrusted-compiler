# 1169 M39 Slice: LASM Cluster Connect-Warning Map Prune

This slice keeps relay warning-throttle state bounded during worker topology churn.

## What changed

1. In relay unhealthy-prune refresh path:
   - clears `connect_warning_next_allowed` when worker snapshot becomes empty,
   - otherwise retains only warning entries for ports present in the current worker snapshot.
2. Existing warning throttle semantics and unhealthy-port cooldown behavior remain unchanged.

## Why

During autoscale churn, worker ports can change over time. Without pruning, warning-throttle state for stale ports can accumulate and add unnecessary map overhead. Pruning keeps warning state aligned with active topology.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
