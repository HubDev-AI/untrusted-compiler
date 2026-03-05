# 1221 M39 Slice: LASM Cluster Status Snapshot Worker-Port Arc Reuse

This slice removes worker-port vector cloning from status snapshot comparisons.

## What changed

1. `LasmClusterStatusSnapshot.worker_ports` now stores `Arc<Vec<u16>>` instead of `Vec<u16>`.
2. Status writer now loads worker ports with `status_worker_ports.load_full()` and passes the shared arc into status-write logic.
3. Snapshot comparison keeps unchanged-snapshot semantics while reusing shared worker-port arcs.

## Why

Typed snapshot comparison removed baseline JSON encoding, but still cloned worker-port vectors each interval. Reusing worker-port arcs eliminates that cloning cost and keeps snapshot comparison deterministic.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
3. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
