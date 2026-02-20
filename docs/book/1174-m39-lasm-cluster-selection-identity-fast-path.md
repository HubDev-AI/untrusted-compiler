# 1174 M39 Slice: LASM Cluster Selection Identity Fast Path

This slice optimizes relay backend selection for healthy steady-state operation.

## What changed

1. `rebuild_lasm_cluster_backend_selection_lookup` now returns two booleans:
   - `has_healthy_backends`,
   - `is_identity_mapping`.
2. Relay worker selection path now fast-paths identity mappings:
   - when `is_identity_mapping == true`, selected backend index is `start_index` directly,
   - otherwise uses lookup table for unhealthy-aware remapping.

## Why

In normal healthy operation, backend selection is identity (round-robin over healthy worker ports). Fast-pathing identity avoids unnecessary lookup-table reads on every connection in steady-state traffic.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
