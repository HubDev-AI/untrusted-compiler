# 1230 M39 Slice: LASM Cluster Cached Selected Worker-Port Count

This slice removes repeated worker-port-count reads from the relay dispatch hot loop.

## What changed

1. Relay worker runtime now keeps `selected_worker_port_count` beside `selected_worker_ports_snapshot`.
2. The cached count updates only when selected worker-port snapshot identity changes.
3. Selection-lookup refresh and backend-index selection now consume this cached count instead of repeatedly reading `selected_worker_ports_snapshot.as_ref().len()`.

## Why

The relay loop performs selection checks for every accepted connection. Caching the selected worker-port count avoids repeated arc/slice length reads and keeps hot-path decisions based on already-known snapshot metadata.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
3. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
