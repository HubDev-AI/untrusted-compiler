# 1032 M39 Slice: LASM Autoscale Saturation Boost

This slice improves cluster autoscale responsiveness by feeding real relay-queue saturation signals into desired-instance calculation.

## What changed

1. Added a relay saturation counter in cluster proxy mode:
   - increments on `TrySendError::Full` (relay queue saturation).
2. Autoscale loop now consumes saturation events each check (`swap(0)`).
3. When saturation is observed, autoscaler boosts desired instance count immediately:
   - boost is bounded by existing controls:
     - `autoscale.max_instances`
     - `--autoscale-scale-up-step`
     - cooldown windows.

## Why

Active-connection heuristics are useful but can lag behind front-proxy queue pressure. Saturation signals provide immediate evidence of backpressure and allow faster scale-up decisions under burst load.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request -- --exact`
- LASM capacity probe with saturated relay constraints (expected fail threshold):
  - `benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh --skip-build --duration 10s --target-requests 450000 --threads 8 --connections 256 --instances 2 --autoscale-max-instances 8 --autoscale-target-connections 128 --cluster-relay-workers 4 --cluster-relay-queue 64 --out benchmark-suite/results/summaries/sec4-lasm-cluster-capacity-probe-saturation-boost.json`
  - observed `344,378` requests (below target, deterministic fail envelope captured).
- LASM capacity probe with tuned relay settings (pass):
  - `benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh --skip-build --duration 10s --target-requests 500000 --threads 8 --connections 256 --instances 4 --autoscale-max-instances 8 --autoscale-target-connections 256 --cluster-relay-workers 32 --cluster-relay-queue 4096 --out benchmark-suite/results/summaries/sec4-lasm-cluster-capacity-probe-saturation-boost-pass.json`
  - observed `622,326` requests (`~61,605 req/s`), peak RSS `10,496 KB`.
