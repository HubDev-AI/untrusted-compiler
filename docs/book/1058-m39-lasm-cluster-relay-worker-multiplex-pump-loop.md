# 1058 M39 Slice: LASM Cluster Relay Worker Multiplex Pump Loop

This slice upgrades LASM cluster proxy relay workers from one-connection-at-a-time handling to multiplexed nonblocking relay pump sets.

## What changed

1. Added `LasmClusterRelayPump` state machine in `compiler/sec4-cli/src/main.rs`:
   - tracks bidirectional buffers and half-close state for one client<->upstream pair,
   - exposes `pump_once()` returning `Progressed`, `Idle`, or `Complete`.
2. Relay worker loop now keeps `Vec<LasmClusterRelayPump>` active set:
   - accepts new client sockets from relay queue,
   - connects upstream backend sockets (same deterministic timeout path),
   - adds new pump states into active set,
   - iterates active set each cycle and advances all connections.
3. Removed head-of-line behavior where each relay worker blocked on a single long-lived keep-alive connection until close.
4. Preserved behavior contracts:
   - same deterministic `503` overload/unavailable responses for no-worker/connect-failure/saturation paths,
   - same saturation event signaling for autoscale pressure,
   - same active-connection accounting semantics.

## Why

Even after previous relay improvements, relay workers could still underutilize available CPU under keep-alive traffic because one worker handled only one proxied connection at a time.

Multiplexing many active relay pairs per worker reduces relay-side scheduling bottlenecks and improves throughput without increasing relay worker thread count.

## Validation

1. Focused cluster command tests:
   - `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
   - `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
2. Short cluster capacity probe (auto relay settings):
   - `benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh --skip-build --duration 20s --target-requests 400000 --threads 8 --connections 256 --instances 4 --autoscale-max-instances 8 --autoscale-target-connections 256 --out benchmark-suite/results/summaries/sec4-lasm-cluster-capacity-probe-latest-multiplex-short.json`

Observed summary for this slice:

- `requests`: `1,474,217`
- `requestsPerSec`: `73,339.27`
- `peakRssKb`: `21,952`
- `p99`: `8.87ms`

Previous auto-mode short-run reference:

- `requests`: `1,446,283`
- `requestsPerSec`: `71,946.59`
- `peakRssKb`: `46,224`
- `p99`: `9.61ms`
