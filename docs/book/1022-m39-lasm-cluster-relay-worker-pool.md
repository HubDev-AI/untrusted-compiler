# 1022 M39 Slice: LASM Cluster Relay Worker Pool (Bounded Proxy Path)

This slice hardens LASM cluster front-proxy execution by removing unbounded relay thread spawning.

## What changed

1. `cmd_run_lasm_cluster` now uses a bounded relay queue (`sync_channel<TcpStream>`) and fixed relay worker pool.
2. Accepted proxy connections are enqueued for relay workers instead of spawning one OS thread per connection.
3. Relay sizing is now resolved from runtime topology:
   - worker count derives from host parallelism + LASM instance bounds,
   - queue capacity derives from target connections per instance and max instance count.
4. Saturated relay queues now return deterministic overload responses:
   - `503` with message `cluster relay saturated`.

## Why

The previous proxy path used an unbounded thread-per-connection relay strategy. Under sustained load this increases scheduler churn and memory pressure, and makes proxy behavior less predictable.

A bounded relay pool gives deterministic backpressure and lowers per-connection overhead in the front layer while preserving existing autoscale and worker recovery behavior.

## Validation

Local load probes (auth-required `/health`, `wrk -t8 -c256 -d10s`) after this change:

- autoscale proxy mode (`--instances 4 --autoscale-max-instances 8`): about `59.9k req/s`,
- fixed reuse-port mode (`--instances 8 --autoscale-max-instances 8`): about `121.1k req/s`.

Targeted command test remained green:

- `run_command_lasm_cluster_mode_serves_request`.

The 1M req/s goal remains open; this slice reduces front-proxy overhead and adds deterministic queue backpressure.
