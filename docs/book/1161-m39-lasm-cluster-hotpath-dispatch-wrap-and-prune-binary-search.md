# 1161 M39 Slice: LASM Cluster Hot Path Dispatch Wrap + Prune Binary Search

This slice reduces per-connection overhead in LASM cluster relay hot paths without changing cluster routing semantics.

## What changed

1. Accept-loop relay dispatch now computes modulo once per listener batch:
   - resolves initial dispatch index from `relay_dispatch_start_base % relay_sender_count`,
   - advances with wrap increment per stream (`+1`, reset to `0` at end) instead of modulo per stream.
2. Worker-port snapshot publish now guards ordering before store:
   - if worker-port order is not ascending, snapshot is sorted before publish.
3. Unhealthy-port prune membership check now uses `binary_search` on the snapshot instead of linear `contains`.

## Why

The proxy accept loop and relay prune run at high frequency under load. Reducing modulo and linear membership checks lowers arithmetic and scan overhead in steady-state loops while preserving the same dispatch/prune decisions.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
