# 1212 M39 Slice: LASM Cluster Accept Next-Index Single Read

This slice trims duplicate lookup work in the multi-relay accept dispatch path.

## What changed

1. Multi-relay accept dispatch now computes `next_dispatch_index` once per accepted stream.
2. The single computed value is reused for:
   - advancing `relay_dispatch_cursor`,
   - choosing fallback dispatch start index when primary sender is full/disconnected.

## Why

The next-index lookup already became precomputed in the prior slice. Reusing the computed value within one dispatch attempt removes duplicate lookup reads in the per-stream hot path while keeping dispatch and fallback ordering unchanged.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
