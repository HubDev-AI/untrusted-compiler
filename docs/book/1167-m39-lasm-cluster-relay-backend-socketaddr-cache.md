# 1167 M39 Slice: LASM Cluster Relay Backend SocketAddr Cache

This slice removes per-connection backend address construction from relay connect hot paths.

## What changed

1. Added helper `rebuild_lasm_cluster_worker_backend_addrs(...)`:
   - rebuilds worker backend `SocketAddr` list from current worker-port snapshot.
2. Relay workers now maintain cached backend address vector:
   - cache is rebuilt only when worker-port snapshot changes,
   - per-connection connect path uses cached backend address by selected backend index.
3. Existing unhealthy map and warning behavior continues to use backend port values from the selected index.

## Why

Per-connection `SocketAddr` construction is unnecessary when backend port snapshot is stable between autoscale/topology changes. Caching backend addresses removes this repeated work from the relay connect path.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
