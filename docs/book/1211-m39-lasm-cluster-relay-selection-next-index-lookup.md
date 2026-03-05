# 1211 M39 Slice: LASM Cluster Relay Selection Next-Index Lookup

This slice reduces per-selection wrap branching in relay backend reservation progression.

## What changed

1. Relay worker selection now keeps a precomputed next-index lookup for current worker cardinality.
2. Lookup is rebuilt only when worker-port count changes.
3. Reservation progression now advances `relay_selection_reservation_next_index` by direct lookup indexing.

## Why

Backend selection reservation progression runs for each accepted relay connection. Precomputing next indexes removes repeated wrap conditionals from that hot path while preserving deterministic round-robin progression.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
