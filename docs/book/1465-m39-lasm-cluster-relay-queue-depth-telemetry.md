# M39: LASM cluster relay queue-depth telemetry

## What changed

Cluster status telemetry now includes live relay queue-depth metrics sampled from relay sender shards.

New status fields:

- `relayQueueDepth` (sum of per-shard queue depth)
- `relayQueueMaxDepth` (max queue depth among relay shards)

Implementation:

- status writer now receives relay sender handles,
- each status interval samples `sender.len()` for each shard,
- sampled aggregate and max depth are emitted in status JSON snapshots.

## Why

Relay saturation counters show failures, but not how close the relay queue is to pressure limits before saturation.

Queue-depth telemetry adds direct backpressure visibility for tuning relay worker count, queue capacity, and accept batching under load.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
