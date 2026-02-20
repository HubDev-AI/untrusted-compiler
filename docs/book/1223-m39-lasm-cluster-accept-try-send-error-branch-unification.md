# 1223 M39 Slice: LASM Cluster Accept Try-Send Error Branch Unification

This slice removes duplicated `try_send` error branches in accept dispatch paths.

## What changed

1. Single-relay accept path now maps `TrySendError::{Full,Disconnected}` through one shared branch into `LasmClusterRelayDispatchError`.
2. Multi-relay accept path now handles `TrySendError` through one shared branch that derives:
   - stream value,
   - `saw_live_sender` flag for fallback classification.
3. Fallback dispatch and existing saturated/unavailable envelope behavior remain unchanged.

## Why

The previous paths duplicated near-identical control flow per error variant. Unifying these branches reduces duplicate hot-path branching/logic while preserving deterministic dispatch semantics.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
3. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
