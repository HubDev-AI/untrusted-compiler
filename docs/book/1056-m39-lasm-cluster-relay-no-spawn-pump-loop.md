# 1056 M39 Slice: LASM Cluster Relay No-Spawn Pump Loop

This slice removes per-connection relay helper thread creation from LASM cluster proxy mode and replaces it with a single-thread nonblocking bidirectional relay loop.

## What changed

1. Reworked `relay_lasm_cluster_connection(...)` in `compiler/sec4-cli/src/main.rs`:
   - removed `try_clone` + per-connection `std::thread::spawn` bridge,
   - added single-thread nonblocking I/O pump for both directions (`client -> upstream`, `upstream -> client`),
   - preserved deterministic half-close behavior (`shutdown(Write)`) once each direction drains.
2. Added bounded idle backoff in the relay loop (`yield_now` + short sleep) to avoid hot spinning when both sockets are temporarily blocked.
3. Left cluster overload behavior unchanged:
   - same `503` deterministic response paths (`cluster relay saturated`, `worker unavailable`, `no healthy workers`),
   - same autoscale/saturation signals and routing selection contracts.

## Why

The previous proxy path still created one extra thread per active proxied connection during relay. Under sustained load this introduced avoidable thread creation and stream-clone overhead in the hot path.

By keeping relay in one worker thread per active connection, this slice reduces that overhead without changing runtime API behavior or envelope contracts.

## Validation

1. Focused command tests:
   - `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
   - `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
2. Capacity probe (same profile as prior short-baseline run):
   - `benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh --skip-build --duration 20s --target-requests 400000 --threads 8 --connections 256 --instances 4 --autoscale-max-instances 8 --autoscale-target-connections 256 --cluster-relay-workers 32 --cluster-relay-queue 4096 --out benchmark-suite/results/summaries/sec4-lasm-cluster-capacity-probe-latest-short.json`

Observed summary after this slice:

- `requests`: `1,417,688`
- `requestsPerSec`: `70,526.20`
- `peakRssKb`: `13,632`
- `p99`: `796.84ms`

Reference previous short run before this slice:

- `requests`: `1,305,339`
- `requestsPerSec`: `64,935.99`
- `peakRssKb`: `10,608`
- `p99`: `863.85ms`
