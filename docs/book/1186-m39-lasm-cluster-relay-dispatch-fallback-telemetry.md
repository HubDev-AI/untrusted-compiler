# 1186 M39 Slice: LASM Cluster Relay Dispatch Fallback Telemetry

This slice adds operator-visible telemetry for preferred-shard dispatch misses.

## What changed

1. Added shared cluster counter `relayDispatchFallbackTotal` (atomic `u64`).
2. Accept loop now increments this counter whenever preferred-shard direct dispatch fails and fallback scanning runs.
3. Cluster status JSON now includes `relayDispatchFallbackTotal`.

## Why

This gives direct visibility into how often relay traffic misses the preferred shard and falls back to scan mode, which is useful when tuning queue/shard sizing and evaluating dispatch hot-path behavior under load.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
