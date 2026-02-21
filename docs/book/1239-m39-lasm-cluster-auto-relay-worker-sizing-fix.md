# 1239 M39 Slice: LASM Cluster Auto Relay-Worker Sizing Fix

This slice corrects auto relay-worker sizing and yields a measurable throughput/latency improvement.

## What changed

1. Replaced relay worker auto-sizing logic that scaled with `target_connections_per_instance`.
2. New default auto-sizing uses bounded instance-based heuristic:
   - `instance_hint = max(min_instances, max_instances)`
   - `relay_hint = 1` for single-instance, otherwise `max(2, ceil(sqrt(instance_hint)))`
   - bounded by host parallelism and hard clamp `1..16`
3. Existing explicit override behavior is unchanged:
   - `--cluster-relay-workers <n>` still takes precedence.

## Why

The previous formula could produce severely over-scaled defaults (for example `256` relay workers for a `4..8` instance config), creating avoidable coordination overhead in relay dispatch. The new bounded heuristic keeps relay-worker defaults proportional to instance scale and host capacity.

## Measurement (same probe envelope)

Probe command family:

- `benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh --duration 20s --target-requests 1000000 --skip-build ...`

Observed results (`/health`, `8t/256c`, `instances=4`, `autoscale-max=8`, auto relay workers):

1. Before fix: `~68,552 req/s`, `p99 11.99ms`
2. After fix: `~75,523 req/s`, `p99 6.41ms`

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
3. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
4. `benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh --duration 20s --target-requests 1000000 --skip-build --out results/summaries/sec4-lasm-cluster-capacity-probe-current-after-auto-tune.json`
