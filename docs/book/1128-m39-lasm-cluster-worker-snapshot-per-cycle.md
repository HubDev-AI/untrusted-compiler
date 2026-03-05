# 1128 M39 Slice: LASM Cluster Worker Snapshot Per Cycle

This slice reduces relay-loop worker-port lookup overhead in LASM cluster mode.

## What changed

1. Updated relay worker scheduling to load worker-port snapshot once per worker cycle.
2. Added per-cycle worker-port references in relay loop:
   - `worker_ports_snapshot`
   - `worker_ports`
   - `worker_port_count`
3. Backend selection now reuses the per-cycle snapshot for each accepted connection in that cycle.
4. Removed per-accepted-connection `relay_worker_ports.load()` calls from the accept hot path.

## Why

The previous relay accept loop loaded worker-port snapshot from `ArcSwap` for every accepted connection.

In sustained traffic, that adds unnecessary atomic/snapshot overhead in the hottest path.

Using one snapshot per cycle keeps routing deterministic while reducing per-connection overhead.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
