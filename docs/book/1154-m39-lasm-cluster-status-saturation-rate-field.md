# 1154 M39 Slice: LASM Cluster Status Saturation Rate Field

This slice adds a direct relay saturation-rate signal to cluster status telemetry.

## What changed

1. Added `relaySaturationEventsPerSec` to cluster status JSON payload.
2. Status writer now tracks previous saturation total and sample timestamp.
3. Rate is computed as delta of `relaySaturationEventsTotal` over elapsed status interval seconds.
4. Existing counters (`relaySaturationEventsPending`, `relaySaturationEventsTotal`) remain unchanged.

## Why

Pending/total saturation counters are useful but require manual diffing to understand pressure trends.

Adding a per-second saturation rate makes overload pressure immediately visible for tuning and autoscale diagnosis.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
