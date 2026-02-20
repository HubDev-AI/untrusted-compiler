# 1210 M39 Slice: LASM Cluster Accept Next-Index Lookup

This slice reduces per-request index-wrap branching in the multi-relay accept path.

## What changed

1. Multi-relay accept path now builds a precomputed sender-next-index lookup once at loop setup.
2. Dispatch cursor advancement now uses direct lookup indexing instead of per-request wrap conditionals.
3. Fallback start selection now reuses the same lookup entry for the selected sender index.

## Why

Cursor/fallback index progression is executed for every accepted connection in multi-relay mode. Precomputing the next-index mapping removes repeated wrap branches from the dispatch hot path while preserving round-robin and fallback ordering semantics.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
