# 1131 M39 Slice: LASM Cluster Saturation Counter Batch

This slice reduces relay failure-path atomic overhead in LASM cluster mode.

## What changed

1. Added local per-worker-cycle saturation counters in relay workers:
   - `saturation_events_pending_local`
   - `saturation_events_total_local`
2. Relay failure paths now increment local counters instead of shared atomics directly:
   - `no healthy workers` path
   - backend connect failure path
3. Added per-cycle flush of local counters into shared saturation atomics after accept/pump processing.

## Why

In failure-heavy intervals, per-event atomic updates add unnecessary contention and write traffic in relay worker loops.

Batching increments locally and flushing once per worker cycle keeps deterministic counter semantics while reducing hot-path atomic overhead.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
