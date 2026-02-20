# 1215 M39 Slice: LASM Cluster Fallback Next-Index Iteration

This slice updates fallback relay sender traversal to iterate with the precomputed next-index lookup.

## What changed

1. `dispatch_lasm_cluster_relay_stream_fallback(...)` now accepts `relay_dispatch_next_index_by_sender`.
2. Fallback scan now performs fixed-attempt iteration (`sender_count - 1`) using next-index lookup progression.
3. Split-slice scan arithmetic (`first span` + `remaining span`) was removed.

## Why

Accept-path dispatch already precomputes next sender indexes. Reusing that lookup in fallback traversal removes extra scan arithmetic and keeps sender-attempt ordering deterministic.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
