# 1240 M39 Slice: LASM Cluster Auto Relay-Worker Floor-Sqrt Tuning

This slice tunes the LASM cluster auto relay-worker heuristic to the faster default observed in current capacity probes.

## What changed

1. Updated `lasm_cluster_proxy_worker_count(...)` auto sizing for multi-instance mode:
   - from `max(2, ceil(sqrt(instance_hint)))`
   - to `max(2, floor(sqrt(instance_hint)))`
2. Existing bounds and precedence are unchanged:
   - explicit `--cluster-relay-workers <n>` still overrides auto,
   - host-parallelism cap still applies,
   - hard clamp `1..16` still applies.

## Why

After fixing prior over-scaling (`1239`), probe runs still showed `relay_workers=2` outperforming auto `relay_workers=3` under the standard short cluster probe envelope (`instances=4`, `max_instances=8`). Switching auto to floor-sqrt preserves bounded default behavior while reducing relay-thread contention.

## Measurement (same probe envelope)

Probe command family:

- `benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh --duration 20s --threads 8 --connections 256 --target-requests 1000000 ...`

Observed auto-mode results (`/health`, `8t/256c`, `instances=4`, `autoscale-max=8`):

1. Previous auto heuristic (`ceil(sqrt(...))`): `~75,523 req/s`, `p99 6.41ms`
2. New auto heuristic (`floor(sqrt(...))`): `~77,766 req/s`, `p99 5.35ms`

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
3. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
4. `benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh --project-path examples/lasm-alpha-full --request-path /health --request-header 'Authorization: Bearer token123' --duration 20s --threads 8 --connections 256 --target-requests 1000000 --port 18096 --instances 4 --autoscale-max-instances 8 --autoscale-target-connections 256 --out results/summaries/sec4-lasm-cluster-capacity-probe-auto-floor.json`
