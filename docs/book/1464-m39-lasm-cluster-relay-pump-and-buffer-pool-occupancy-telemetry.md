# M39: LASM cluster relay pump and buffer-pool occupancy telemetry

## What changed

LASM cluster relay workers now publish aggregate hot-path occupancy counters, and status JSON exposes them.

New status fields:

- `relayPumpConnections`
- `relayBufferPoolEntries`

Implementation changes:

- relay worker loop now accepts shared atomics for aggregate relay-pump and buffer-pool occupancy,
- each worker applies delta-based `fetch_add`/`fetch_sub` updates only when local counts change,
- worker exit path flushes local contributions back to zero,
- status writer includes both counters in each cluster snapshot.

## Why

Relay tuning needed direct visibility into live relay concurrency and reusable-buffer occupancy, not only static size limits.

These counters make it possible to correlate saturation/latency behavior with actual relay hot-path occupancy during load runs.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
