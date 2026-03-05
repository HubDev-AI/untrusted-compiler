# 1187 M39 Slice: LASM Cluster Per-Accept-Worker Local Dispatch Cursors

This slice removes one shared atomic from the cluster accept dispatch hot path.

## What changed

1. `run_lasm_cluster_accept_loop` no longer takes a shared dispatch `AtomicUsize`.
2. Each accept loop now keeps a local `relay_dispatch_cursor` seeded by worker index.
3. Multi-relay dispatch start index is derived from the local cursor, then cursor advances by accepted batch length.
4. Shared fallback telemetry (`relayDispatchFallbackTotal`) remains unchanged.

## Why

Per-worker local cursors preserve deterministic round-robin-per-worker dispatch behavior while removing shared atomic contention between accept workers.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
