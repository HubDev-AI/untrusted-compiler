# 1229 M39 Slice: LASM Cluster Accept/Fallback Direct Next-Index Helper

This slice removes another next-index lookup vector from LASM cluster dispatch paths.

## What changed

1. Added shared wrapped-index helper `lasm_cluster_next_index_wrapped(index, count)`.
2. Multi-relay accept-loop dispatch now advances sender cursor through the helper instead of `relay_dispatch_next_index_by_sender` lookup vector.
3. Fallback relay scan now uses the same helper for sender iteration, removing its dependency on sender next-index lookup slices.

## Why

The accept loop previously allocated/prebuilt a sender next-index vector and read it in both primary multi-relay dispatch and fallback scan loops. Direct wrapped arithmetic keeps identical scan order semantics while removing setup and lookup overhead.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
3. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
