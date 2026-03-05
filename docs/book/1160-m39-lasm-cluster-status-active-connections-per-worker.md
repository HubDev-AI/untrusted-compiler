# 1160 M39 Slice: LASM Cluster Status Active Connections Per Worker

This slice adds per-worker load-density visibility to cluster status telemetry.

## What changed

1. Added `activeConnectionsPerWorker` to cluster status JSON payload.
2. Status writer now computes:
   - `activeConnectionsPerWorker = activeConnections / workerCount` (or `0.0` when `workerCount == 0`).
3. Existing active connection and worker count fields remain unchanged.

## Why

Raw connection count and worker count are useful but still require manual division to reason about load density.

Emitting per-worker active connection density directly improves operational readability for autoscale and tuning decisions.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
