# 1148 M39 Slice: LASM Cluster Relay Dispatch Slice Iteration

This slice tightens relay queue-shard sender traversal in the dispatch hot path.

## What changed

1. Updated `dispatch_lasm_cluster_relay_stream` to split sender slices at `start_index`.
2. Replaced per-iteration modulo/index sender lookup with ordered split-slice iteration (`tail` then `head`).
3. Kept existing sender fallback semantics intact:
   - first `try_send` success returns immediately,
   - `Full` keeps searching,
   - all-disconnected returns `Unavailable`, otherwise `Saturated`.

## Why

Dispatch runs for every accepted socket in proxy-cluster mode.

Removing repeated modulo/index arithmetic in the sender loop reduces per-dispatch hot-path overhead without changing routing behavior.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
3. `benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh --duration 20s --target-requests 600000 --skip-build --out benchmark-suite/results/m39-cluster-probe-dispatch-iter-20260220-185613.json`
