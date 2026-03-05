# M39: LASM Autoscale Reusable Port Telemetry Store Elision

Date: 2026-02-22  
Milestone: M39 (scaling/runtime hot-path overhead reduction)

## What Changed

- Added reusable-port telemetry change detection in autoscale loop:
  - introduced helper that stores `reusablePortsCount` atomic only when the value changed,
  - tracked last published reusable-port count across loop iterations.
- Applied helper in both autoscale reusable-port publication points:
  - maintenance/pre-action loop path,
  - post spawn/apply reconciliation path.

## Why

Autoscale loop previously stored `reusablePortsCount` every maintenance tick even when the reusable-port pool size was unchanged. This created avoidable atomic-store churn under steady-state traffic.

## Result

- `reusablePortsCount` telemetry still updates deterministically when port pool size changes.
- Steady-state ticks avoid redundant atomic writes when reusable-port count is stable.
- Cluster status and scaling behavior remain unchanged; this is overhead reduction only.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
