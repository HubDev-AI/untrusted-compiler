# 1153 M39 Slice: LASM Cluster Status Relay Queue Capacity Fields

This slice extends cluster status telemetry with relay queue sizing details.

## What changed

1. Added `relayQueueCapacity` to cluster status JSON payload.
2. Added `relayQueueShardCapacity` to cluster status JSON payload.
3. Wired resolved runtime values from cluster setup into status writer output.
4. Kept status write cadence and existing fields unchanged.

## Why

Operators tuning relay throughput and saturation behavior need queue sizing visibility along with worker counts.

Exposing queue totals and per-shard capacity directly in status JSON removes guesswork when comparing tuning runs.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
