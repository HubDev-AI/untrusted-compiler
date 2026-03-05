# 1202 M39 Slice: LASM Cluster Fallback Scan Direct Prewrapped Start

This slice removes a redundant runtime branch from relay fallback scanning.

## What changed

1. `dispatch_lasm_cluster_relay_stream_fallback` now uses:
   - `scan_start_index = start_index_wrapped`
2. Existing debug invariant (`start_index_wrapped < sender_count`) remains in place.

## Why

Fallback start indexes are already wrapped at call sites. Removing repeated runtime normalization from fallback dispatch trims one branch in this path while preserving behavior.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
