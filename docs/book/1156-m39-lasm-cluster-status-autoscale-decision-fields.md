# 1156 M39 Slice: LASM Cluster Status Autoscale Decision Fields

This slice exposes autoscale decision context directly in cluster status telemetry.

## What changed

1. Added status payload fields:
   - `autoscaleDesiredInstances`
   - `autoscaleLastSaturationEvents`
   - `autoscaleLastDynamicBoostStep`
2. Added shared autoscale telemetry atomics initialized during cluster runtime startup.
3. Autoscale loop now updates these telemetry fields on each evaluation cycle.
4. Status writer reads and emits the latest autoscale decision values with existing status fields.

## Why

Autoscale decisions were previously internal to the maintenance loop.

Surfacing latest desired instance count, recent saturation-event volume, and dynamic boost step makes scaling behavior observable without inspecting logs or reproducing internals.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
