# 1140 M39 Slice: LASM Cluster Sharded Relay Queue And Batched Listener Dispatch

This slice removes relay enqueue contention in LASM cluster mode and batches listener-side dispatch.

## What changed

1. Replaced the single relay queue with per-worker sharded relay queues.
2. Added deterministic relay dispatch helper that round-robins incoming sockets across shards and distinguishes `saturated` vs `unavailable` outcomes.
3. Switched listener accept path to nonblocking batched intake (`cluster_relay_accept_batch_max`) before dispatching sockets to relay shards.
4. Kept existing overload/error envelopes unchanged (`cluster relay saturated`, `cluster relay unavailable`) while preserving saturation counters.

## Why

A single shared relay queue creates avoidable contention between listener enqueue and relay workers under load. Sharding queues by relay worker reduces that hotspot and keeps queue operations local.

Batching listener accept/dispatch reduces per-connection scheduling churn, so each loop can move multiple accepted sockets through dispatch before yielding.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
3. `benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh --duration 20s --target-requests 600000 --skip-build --out benchmark-suite/results/summaries/sec4-lasm-cluster-capacity-probe-sharded-queue.json`
