# M39: LASM Autoscale Telemetry Store Elision

Date: 2026-02-22  
Milestone: M39 (cluster autoscale runtime overhead reduction)

## What Changed

- Extended autoscale-loop change-detection telemetry writes beyond reusable-port count.
- Added last-value tracking and store-if-changed helpers for:
  - `autoscaleLastDesiredInstances`
  - `autoscaleLastSaturationEvents`
  - `autoscaleLastDynamicBoostStep`
  - `autoscaleScaleUpCooldownRemainingMs`
  - `autoscaleScaleDownCooldownRemainingMs`
- Autoscale loop now writes these shared atomics only when values actually change.

## Why

Autoscale loop was writing multiple telemetry atomics every maintenance tick even when values were stable. This created avoidable background atomic-store churn during steady-state operation.

## Result

- Telemetry semantics and reported values remain unchanged.
- Steady-state autoscale loop performs fewer redundant atomic writes.
- Runtime behavior for scaling decisions remains intact.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
