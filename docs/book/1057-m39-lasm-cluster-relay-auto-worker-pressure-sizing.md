# 1057 M39 Slice: LASM Cluster Relay Auto Worker Pressure Sizing

This slice improves cluster-proxy default relay worker sizing so auto mode better matches configured connection pressure.

## What changed

1. Updated `lasm_cluster_proxy_worker_count(...)` in `compiler/sec4-cli/src/main.rs`.
2. When `--cluster-relay-workers` is not explicitly set:
   - worker count now uses `target_connections_per_instance` as a floor (plus `min_instances` and host parallelism floors),
   - auto-mode worker count is bounded by an adaptive cap derived from `max_instances * target_connections_per_instance`,
   - adaptive cap is clamped to `128..512` to prevent unbounded thread growth.

## Why

After removing per-connection relay helper thread spawn, default auto relay worker sizing was still too conservative for keep-alive-heavy proxy workloads. In practical cluster probes this left significant connection pressure on the queue path unless operators explicitly tuned `--cluster-relay-workers`.

This change makes auto mode useful out of the box for the common high-connection profile.

## Validation

1. Focused cluster command tests:
   - `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
   - `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
2. Short probe with pinned relay settings (reference):
   - `benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh --skip-build --duration 20s --target-requests 400000 --threads 8 --connections 256 --instances 4 --autoscale-max-instances 8 --autoscale-target-connections 256 --cluster-relay-workers 32 --cluster-relay-queue 4096 --out benchmark-suite/results/summaries/sec4-lasm-cluster-capacity-probe-latest-short.json`
3. Short probe in auto relay mode (new behavior):
   - `benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh --skip-build --duration 20s --target-requests 400000 --threads 8 --connections 256 --instances 4 --autoscale-max-instances 8 --autoscale-target-connections 256 --out benchmark-suite/results/summaries/sec4-lasm-cluster-capacity-probe-latest-adaptive-short.json`

Observed auto-mode summary after this slice:

- `requests`: `1,446,283`
- `requestsPerSec`: `71,946.59`
- `peakRssKb`: `46,224`
- `p99`: `9.61ms`
