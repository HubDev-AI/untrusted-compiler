# 1151 M39 Slice: LASM Cluster Unhealthy Prune Interval Throttle

This slice reduces relay-loop churn from unhealthy-port pruning.

## What changed

1. Added bounded unhealthy prune interval constant:
   - `LASM_CLUSTER_UNHEALTHY_PRUNE_INTERVAL_MS = 2`
2. Relay worker unhealthy-port prune now runs only when the interval elapses.
3. When unhealthy entries are absent, prune scheduling is disabled.
4. Worker-port snapshot loading for backend selection remains on-demand per accepted socket.

## Why

Previously, any non-empty unhealthy map forced prune+snapshot work every relay loop cycle.

Throttling prune cadence keeps unhealthy retention semantics while reducing unnecessary prune-side snapshot churn in the hot loop.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
3. `benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh --duration 20s --target-requests 600000 --skip-build --out benchmark-suite/results/m39-cluster-probe-unhealthy-prune-interval2-20260220-191734.json`
