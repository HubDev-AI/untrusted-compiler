# 1163 M39 Slice: LASM Cluster Relay Buffer Pool Reuse

This slice adds bounded relay-buffer reuse in worker threads to reduce repeated allocation churn under connection-heavy traffic.

## What changed

1. Extended `LasmClusterRelayPump` with reusable buffer construction path:
   - `new_with_buffers(...)` now accepts externally provided relay buffers,
   - `into_buffers()` returns buffers when a pump completes/fails.
2. Relay worker loop now maintains a bounded local buffer pool:
   - pool size bound: `max(64, relay_accept_batch_max * 4)`,
   - on successful connect: reuse pooled buffers when available; otherwise allocate fresh buffers,
   - on pump completion/failure: return buffers to pool when under bound.

## Why

Without pooling, each proxied connection allocates two relay buffers. Reusing buffers across completed connections reduces per-connection allocation/deallocation overhead while keeping deterministic memory bounds.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
