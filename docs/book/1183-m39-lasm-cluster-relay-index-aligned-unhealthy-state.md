# 1183 M39 Slice: LASM Cluster Relay Index-Aligned Unhealthy State

This slice removes hash-map state from relay backend-health refresh paths.

## What changed

1. Relay workers now store backend unhealthy/connect-warning timing state as index-aligned vectors:
   - `unhealthy_ports_until_by_index: Vec<Option<Instant>>`
   - `connect_warning_next_allowed_by_index: Vec<Option<Instant>>`
2. On worker-topology snapshot changes, relay workers remap index-aligned state from previous port snapshots into new snapshots.
3. Snapshot remap now reuses vector ownership with `std::mem::take(...)` instead of cloning relay unhealthy/warning vectors before remap.
4. Backend-selection lookup rebuild now consumes index-aligned unhealthy state directly (plus unhealthy-count), instead of per-port hash lookups.
5. Connect-failure cooldown updates now write unhealthy/warning state by selected backend index.

## Why

This removes hash lookup overhead from relay selection rebuild and connect-failure cooldown checks in the hot path while preserving deterministic unhealthy pruning/remap behavior across topology changes.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
