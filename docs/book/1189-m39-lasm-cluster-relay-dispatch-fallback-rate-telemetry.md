# 1189 M39 Slice: LASM Cluster Relay Dispatch Fallback Rate Telemetry

This slice extends fallback dispatch telemetry with a per-second rate signal.

## What changed

1. Cluster status JSON now includes `relayDispatchFallbackPerSec`.
2. Status writer computes this field from per-interval deltas of `relayDispatchFallbackTotal`.
3. Existing `relayDispatchFallbackTotal` remains unchanged.

## Why

Total fallback counts are useful for long-run visibility, while per-second rates help operators detect current fallback pressure and correlate spikes with saturation/queue behavior during load tuning.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
