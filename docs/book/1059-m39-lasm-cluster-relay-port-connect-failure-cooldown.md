# 1059 M39 Slice: LASM Cluster Relay Port Connect-Failure Cooldown

This slice hardens LASM cluster relay behavior under worker churn by adding per-relay-worker temporary cooldown tracking for backend ports that fail connect attempts.

## What changed

1. Added relay helper:
   - `lasm_cluster_backend_connect_cooldown()` (`500ms`).
2. Each relay worker thread now keeps a local `HashMap<u16, Instant>` of temporarily unhealthy backend ports.
3. On backend connect failure:
   - failed port is marked unhealthy until `now + cooldown`,
   - existing deterministic `503` unavailable response path remains unchanged.
4. Backend selection logic now has two branches:
   - healthy fast path when no unhealthy ports are tracked (direct round-robin selection),
   - cooldown-aware selection that skips unhealthy ports when failures have been observed.

## Why

Without cooldown tracking, relay workers can repeatedly hit the same transiently unavailable backend port during short worker churn windows, producing unnecessary repeated connect failures before autoscale/maintenance reconciliation catches up.

Cooldown reduces that retry storm while preserving the same external response contracts.

## Validation

1. Focused cluster command tests:
   - `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
   - `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
2. Short cluster probe (healthy path sanity):
   - `benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh --skip-build --duration 20s --target-requests 400000 --threads 8 --connections 256 --instances 4 --autoscale-max-instances 8 --autoscale-target-connections 256 --out benchmark-suite/results/summaries/sec4-lasm-cluster-capacity-probe-latest-port-cooldown-fastpath-short.json`

Observed healthy-profile summary:

- `requests`: `1,457,249`
- `requestsPerSec`: `72,490.70`
- `peakRssKb`: `21,888`
- `p99`: `9.54ms`

Note: this slice is resilience-focused (reduced repeated retries during failure churn), not a throughput-targeting change.
