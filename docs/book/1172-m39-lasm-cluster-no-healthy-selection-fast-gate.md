# 1172 M39 Slice: LASM Cluster No-Healthy Selection Fast Gate

This slice cuts backend-selection overhead when relay workers are in a known no-healthy state.

## What changed

1. `rebuild_lasm_cluster_backend_selection_lookup` now returns a boolean indicating whether at least one healthy backend exists.
2. Relay worker state now tracks `selection_has_healthy_backends` from lookup rebuilds.
3. Per-connection selection path now short-circuits when `selection_has_healthy_backends == false`:
   - skips selection-counter reservation/index progression,
   - keeps existing no-healthy response behavior (`503`, `no healthy workers`).

## Why

When all workers are unhealthy, selection lookup is known to be empty. Continuing selection-counter/index work in that state is wasted hot-path overhead. Fast-gating removes that cost during outage windows.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
