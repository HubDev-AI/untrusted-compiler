# 1231 M39 Slice: LASM Cluster Worker-Port Snapshot Replace Handoff

This slice reduces Arc clone churn when relay workers refresh selected worker-port snapshots.

## What changed

1. Relay worker snapshot refresh paths now use `std::mem::replace` to swap in new selected worker-port snapshots.
2. Previous selected snapshots are moved into remap inputs directly instead of cloning old snapshot arcs.
3. Prune path keeps per-iteration reuse by storing `Arc::clone(&selected_worker_ports_snapshot)` for the local snapshot cache after refresh.

## Why

Worker-port snapshot refresh previously cloned both old and new arcs around remap/rebuild paths. `std::mem::replace` preserves behavior while reducing reference-count updates and unnecessary clone churn in a frequently executed runtime loop.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
3. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
