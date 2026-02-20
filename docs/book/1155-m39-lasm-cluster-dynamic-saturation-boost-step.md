# 1155 M39 Slice: LASM Cluster Dynamic Saturation Boost Step

This slice improves autoscale reaction under relay saturation bursts.

## What changed

1. Autoscale saturation handling now computes `dynamic_boost_step` from saturation-event batch volume per evaluation cycle.
2. `dynamic_boost_step` replaces fixed saturation boost size in both:
   - saturation-driven `desired` target bump,
   - scale-up step budget for the current evaluation.
3. Existing guardrails remain unchanged:
   - autoscale cooldowns,
   - max-instance cap,
   - baseline desired-instance computation from active connections.

## Why

Fixed saturation boost size reacts slowly when saturation arrives in larger bursts.

Scaling boost size with observed saturation volume improves scale-up responsiveness while keeping deterministic bounds.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
3. `benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh --duration 20s --target-requests 600000 --skip-build --out benchmark-suite/results/m39-cluster-probe-dynamic-saturation-boost-20260220-194803.json`
