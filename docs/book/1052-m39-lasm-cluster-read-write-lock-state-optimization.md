# 1052 M39 Slice: LASM Cluster Read/Write Lock State Optimization

This slice further reduces relay-path lock contention by changing LASM cluster state synchronization from `Mutex` to `RwLock`.

## What changed

1. In `compiler/sec4-cli/src/main.rs`, shared cluster state for proxy mode now uses:
   - `Arc<RwLock<LasmClusterState>>`
   instead of `Arc<Mutex<LasmClusterState>>`.
2. Relay worker dispatch now acquires read access only when selecting a backend worker port.
3. Maintenance/autoscale loop acquires write access for:
   - dead-worker pruning,
   - min-instance recovery,
   - scale up/down worker mutations.
4. Final shutdown path now acquires write access before stopping workers.

## Why

After moving prune/recovery out of per-connection relay handling, relay workers were still serialized behind a single mutex even for read-only worker selection. Using a read/write lock allows concurrent relay selection while preserving single-writer safety for scaling mutations.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`

## Notes

- This is a concurrency hot-path optimization slice, not a full throughput target closure.
- Additional proxy/runtime IO-path optimization is still required for the 1M req/s goal.
