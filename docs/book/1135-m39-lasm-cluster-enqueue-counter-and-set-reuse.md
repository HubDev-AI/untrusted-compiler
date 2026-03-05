# 1135 M39 Slice: LASM Cluster Enqueue Counter and Set Reuse

This slice removes avoidable per-request/per-cycle overhead in LASM cluster relay hot paths.

## What changed

1. Listener enqueue path now increments `active_connections` only on successful relay queue enqueue.
2. Removed add/sub counter churn on relay queue rejection paths (`Full`/`Disconnected`).
3. Relay worker unhealthy-membership pruning now reuses one mutable active-port set across cycles.
4. Replaced per-cycle fresh active-port set allocation with clear+extend reuse.

## Why

Under saturation, listener queue rejection previously performed an unnecessary increment followed by decrement on `active_connections` for requests that were never enqueued.

In relay workers, active-port membership pruning previously allocated a new set each cycle when unhealthy entries existed.

Both changes reduce hot-path atomic/allocation churn while preserving deterministic runtime behavior.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
