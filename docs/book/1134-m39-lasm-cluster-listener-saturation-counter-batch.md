# 1134 M39 Slice: LASM Cluster Listener Saturation Counter Batch

This slice reduces accept-loop atomic churn under saturation in LASM cluster mode.

## What changed

1. Added shared saturation-counter flush helper:
   - `flush_lasm_cluster_saturation_counters(...)`
2. Added bounded listener-side local saturation counters for queue-full events.
3. Accept loop now batches saturation atomic updates with configurable flush threshold:
   - `LASM_CLUSTER_SATURATION_COUNTER_FLUSH_BATCH` (current value: `8`)
4. Relay worker saturation flush now also uses the shared helper for consistency.

## Why

When relay queue is full, listener path can emit many saturation events rapidly.

Per-event atomic increments in that path add avoidable overhead. Local batching preserves deterministic counter behavior while reducing write contention.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
