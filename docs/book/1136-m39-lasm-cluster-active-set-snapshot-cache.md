# 1136 M39 Slice: LASM Cluster Active Set Snapshot Cache

This slice reduces repeated set rebuild work in relay unhealthy-membership pruning.

## What changed

1. Added relay-local active worker-port snapshot cache:
   - `active_worker_ports_snapshot: Option<Arc<Vec<u16>>>`
2. Active worker-port set rebuild now occurs only when worker snapshot identity changes:
   - uses `Arc::ptr_eq` against cached snapshot
3. Preserved existing unhealthy pruning semantics while avoiding per-cycle set repopulation when worker membership is unchanged.

## Why

After set-reuse introduction, relay workers still repopulated the active-port set on each cycle with unhealthy entries.

Caching by snapshot identity removes that repeated work for steady-state worker sets while keeping deterministic behavior across autoscale worker changes.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
