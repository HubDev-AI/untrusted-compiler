# 1030 M39 Slice: LASM Autoscale Step-Window Controls

This slice extends LASM autoscale hysteresis with bounded worker-count changes per autoscale tick.

## What changed

1. Added new `sec4 run` LASM flags:
   - `--autoscale-scale-up-step` (default `2`)
   - `--autoscale-scale-down-step` (default `1`)
2. Cluster autoscaler now caps scale movement per check:
   - scale-up target per tick is bounded by `current + scale_up_step`,
   - scale-down target per tick is bounded by `current - scale_down_step`.
3. Added deterministic validation + guards:
   - both flags require values `>= 1`,
   - both flags are LASM-only.
4. Extended LASM cluster capacity probe tooling to accept and forward step flags:
   - `benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh`
   - dry-run contract test updated accordingly.

## Why

Cooldown windows reduce oscillation over time, but large instant jumps can still cause process churn. Step-window limits make autoscaling smoother and more predictable under bursty traffic.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 --test commands run_command_rejects_autoscale_scale_up_step_with_c_backend -- --exact`
- `cargo test -p sec4 --test commands run_command_rejects_zero_autoscale_scale_down_step -- --exact`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request -- --exact`
- `benchmark-suite/scripts/test_run_lasm_cluster_capacity_probe.sh`
- probe run with step flags:
  - `benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh --skip-build --duration 10s --target-requests 500000 --threads 8 --connections 256 --instances 4 --autoscale-max-instances 8 --autoscale-target-connections 256 --autoscale-scale-up-step 2 --autoscale-scale-down-step 1 --cluster-relay-workers 32 --cluster-relay-queue 4096 --out benchmark-suite/results/summaries/sec4-lasm-cluster-capacity-probe-step-window.json`
  - pass with `634,070` requests (`~62,750 req/s`), peak RSS about `10,352 KB`.
