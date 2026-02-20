# 1157 M39 Slice: LASM Cluster Saturation-Priority Autoscale Evaluation

This slice improves saturation reaction time in cluster autoscaling.

## What changed

1. Autoscale loop now checks pending saturation events before enforcing `autoscale_check_ms` interval gate.
2. When saturation is pending, autoscale evaluation proceeds immediately at maintenance-loop cadence.
3. When saturation is absent, existing `autoscale_check_ms` cadence remains unchanged.

## Why

Previously, saturation-driven scaling could wait up to the next autoscale check interval even when saturation was already present.

Allowing saturation to bypass that gate reduces reaction latency under overload bursts without changing steady-state evaluation behavior.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
3. `benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh --duration 20s --target-requests 600000 --skip-build --out benchmark-suite/results/m39-cluster-probe-saturation-priority-eval-20260220-195306.json`
