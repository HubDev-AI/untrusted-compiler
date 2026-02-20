# 1171 M39 Slice: LASM Cluster Selection Lookup Scratch-Mask Reuse

This slice removes temporary allocation churn from relay selection lookup rebuilds.

## What changed

1. `rebuild_lasm_cluster_backend_selection_lookup` now accepts a mutable healthy-mask scratch buffer.
2. Relay workers now keep a reusable `selection_healthy_mask: Vec<bool>` alongside selection lookup state.
3. Lookup rebuild uses `clear + resize` on reusable mask buffer instead of allocating a fresh healthy-mask vector per rebuild.

## Why

Topology/health changes can trigger repeated lookup rebuilds. Reusing the mask scratch buffer removes avoidable short-lived allocations on rebuild paths while keeping behavior unchanged.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
