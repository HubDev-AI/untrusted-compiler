# 1266 M39 Slice: LASM Cluster Status Relay Live-Shard Telemetry

This slice adds relay-shard liveness visibility to LASM cluster status snapshots.

## What changed

1. Added `relay_live_sender_count` to `LasmClusterStatusSnapshot` and serialized payload (`relayLiveSenderCount`).
2. Added shared relay-live counter plumbing from accept loop into status writer path.
3. Accept loop now reports relay-live count transitions into the shared counter during degraded/disconnect dispatch flow.
4. Extended `run_command_lasm_cluster_status_json_skips_unchanged_snapshots` assertions to require `relayLiveSenderCount` presence.

## Why

Throughput and fallback counters show pressure signals, but they do not show relay-shard liveness directly. Emitting live-shard count in status snapshots improves operational visibility and helps correlate dispatch degradation with shard disconnect behavior during saturation tuning.

## Validation

1. `rustfmt compiler/sec4-cli/src/main.rs compiler/sec4-cli/tests/commands.rs`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
3. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
