# 1191 M39 Slice: LASM Cluster Split Single vs Multi Relay Accept Dispatch

This slice removes per-stream sender-count branching from the LASM cluster accept dispatch hot path.

## What changed

1. `run_lasm_cluster_accept_loop` now runs separate accept-dispatch paths for:
   - single-relay sender mode, and
   - multi-relay sender mode.
2. The multi-relay path keeps round-robin cursor progression and fallback dispatch behavior unchanged.
3. Dispatch result handling (success/saturated/unavailable) is centralized in `handle_lasm_cluster_accept_dispatch_result`.

## Why

The previous implementation evaluated single-vs-multi sender branching for each accepted stream. Splitting dispatch paths once per loop iteration removes that repeated branch on the innermost accept path while preserving operational behavior.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
