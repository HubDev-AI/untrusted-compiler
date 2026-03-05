# 1228 M39 Slice: LASM Cluster Direct Round-Robin Next Index

This slice simplifies relay round-robin index progression in the cluster hot path.

## What changed

1. Removed `relay_selection_next_index_by_worker` vector from relay worker dispatch state.
2. Round-robin now advances with direct wrapped arithmetic (`start_index + 1`, wrap to `0`).
3. Reservation counter and selection lookup semantics remain unchanged for multi-backend clusters.

## Why

The previous path rebuilt a next-index lookup vector when worker-port count changed and read that vector on every dispatch step. Direct wrapped arithmetic keeps identical ordering while removing vector rebuild/read overhead in a high-frequency path.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
3. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
