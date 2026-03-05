# 1143 M39 Slice: LASM Cluster Accept Worker Pool

This slice parallelizes front-proxy connection intake/dispatch in LASM cluster mode.

## What changed

1. Added `run_lasm_cluster_accept_loop(...)` as a shared accept+dispatch loop for cluster proxy intake workers.
2. Added support for parallel accept workers via listener socket cloning (`TcpListener::try_clone`).
3. Added env-controlled intake concurrency: `SEC4_RT_LASM_CLUSTER_ACCEPT_WORKERS`.
4. Set deterministic default accept worker count to `min(4, relay_workers)` with bounded override clamp (`1..=16`, and never above relay worker count).
5. Added `relayAcceptWorkers` to cluster status JSON payloads for runtime observability.

## Why

Cluster proxy intake was running on a single accept loop. Under sustained load, that can serialize front-door socket acceptance and shard dispatch even when relay workers are available.

Parallel accept workers allow intake and queue dispatch to proceed concurrently while keeping existing relay worker pool behavior and deterministic error envelopes unchanged.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
3. `benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh --duration 20s --target-requests 600000 --skip-build --out benchmark-suite/results/summaries/sec4-lasm-cluster-capacity-probe-accept-workers-default4.json`
