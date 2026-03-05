# 1158 M39 Slice: LASM Cluster Saturation-Priority Sleep Interval

This slice reduces autoscale-loop reaction latency during saturation.

## What changed

1. Added a saturation-priority autoscale sleep interval:
   - `saturation_priority_interval_ms = min(maintenance_interval_ms, 100)`
2. Autoscale loop now chooses sleep cadence per cycle:
   - if saturation is pending and autoscale is enabled: sleep priority interval,
   - otherwise: sleep baseline maintenance interval.
3. Existing autoscale decision logic and bounds are unchanged.

## Why

Even with saturation-priority evaluation gates, a long maintenance sleep can delay when those evaluations happen.

Shortening sleep cadence only while saturation is pending improves response time to overload while avoiding extra churn in steady state.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
3. `benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh --duration 20s --target-requests 600000 --skip-build --out benchmark-suite/results/m39-cluster-probe-saturation-priority-interval-20260220-195544.json`
