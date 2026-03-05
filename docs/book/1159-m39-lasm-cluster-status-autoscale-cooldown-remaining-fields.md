# 1159 M39 Slice: LASM Cluster Status Autoscale Cooldown Remaining Fields

This slice adds autoscale cooldown state visibility to cluster status telemetry.

## What changed

1. Added status payload fields:
   - `autoscaleScaleUpCooldownRemainingMs`
   - `autoscaleScaleDownCooldownRemainingMs`
2. Added shared cooldown telemetry atomics in cluster runtime.
3. Autoscale loop now updates cooldown remaining values from `last_scale_up_at` / `last_scale_down_at` each cycle.
4. Status writer now emits the latest cooldown remaining values with existing autoscale telemetry fields.

## Why

Operators previously saw desired/scaling-intent fields but not cooldown lockout state.

Exposing remaining cooldown milliseconds makes scaling timing behavior directly observable during tuning and incident diagnosis.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
