# 1142 M39 Slice: LASM Cluster Active-Connection Atomic Batching

This slice reduces active-connection counter atomic churn in LASM cluster hot paths.

## What changed

1. Added helper flush functions for batched active-connection increments/decrements.
2. Listener accept path now accumulates successful shard-enqueue count per batch and applies one `active_connections.fetch_add(...)` per batch.
3. Relay worker path now accumulates completion/failure decrements locally and applies one `active_connections.fetch_sub(...)` per worker loop cycle.
4. Existing saturation counters and unavailable/saturated response semantics remain unchanged.

## Why

Before this slice, the cluster path updated `active_connections` atomically on every single enqueue and every single completion/failure event.

Batching those updates keeps behavior deterministic but lowers atomic pressure in listener and relay worker hot loops.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
3. `benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh --duration 20s --target-requests 600000 --skip-build --out benchmark-suite/results/summaries/sec4-lasm-cluster-capacity-probe-active-counter-batch.json`
