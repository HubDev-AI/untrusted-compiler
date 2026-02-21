# 1227 M39 Slice: LASM Cluster Single-Backend Selection Fast Path

This slice optimizes relay backend selection when only one worker backend is available.

## What changed

1. Relay backend selection now short-circuits to backend index `0` when `worker_port_count == 1`.
2. In that single-backend case, relay worker loop skips reservation-counter `fetch_add` and next-index rotation bookkeeping.
3. Multi-backend behavior is unchanged: reservation + lookup flow still runs for `worker_port_count > 1`.

## Why

The previous path still paid reservation/rotation overhead even when only one backend existed. Single-backend dispatch does not need round-robin state, so this removes unnecessary atomic/bookkeeping work from a high-frequency loop.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
3. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
