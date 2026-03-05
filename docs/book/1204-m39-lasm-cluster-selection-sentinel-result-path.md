# 1204 M39 Slice: LASM Cluster Selection Sentinel Result Path

This slice simplifies relay backend-selection result handling.

## What changed

1. Relay backend selection now carries selection results as `usize` with sentinel `LASM_CLUSTER_SELECTION_LOOKUP_NONE`.
2. Removed per-connection `Option<usize>` construction in selection result path.
3. No-healthy path now checks sentinel directly before returning unavailable response.

## Why

The selection path already uses sentinel-based lookup tables. Carrying sentinel values through the result path avoids extra per-connection option wrapping while preserving existing behavior.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
