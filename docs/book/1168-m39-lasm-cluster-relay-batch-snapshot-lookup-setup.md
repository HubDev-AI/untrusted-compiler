# 1168 M39 Slice: LASM Cluster Relay Batch Snapshot/Lookup Setup

This slice trims repeated per-connection setup work inside relay worker batch processing.

## What changed

1. Relay worker batch loop now initializes snapshot/lookup state only when needed:
   - when batch-local `worker_ports_snapshot` is missing, or
   - when selection lookup is marked dirty.
2. Per-connection path now uses already prepared batch state:
   - avoids repeating snapshot pointer checks and lookup-refresh gating on every accepted connection in the same batch.

## Why

Accepted connections in a relay batch typically share the same worker topology snapshot. Re-running snapshot/lookup guard plumbing for every connection adds avoidable overhead. Moving setup to once-per-batch keeps hot-path connection handling leaner.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
