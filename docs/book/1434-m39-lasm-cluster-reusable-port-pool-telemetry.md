# M39: LASM Cluster Reusable Port Pool Telemetry

Date: 2026-02-22  
Milestone: M39 (scaling/runtime observability hardening)

## What Changed

- Added reusable port pool telemetry for LASM cluster runtime:
  - autoscale loop now publishes `reusable_ports.len()` into shared atomics,
  - status writer now includes `reusablePortsCount` in cluster status JSON.
- Added command integration assertion to lock presence of the new status field.

## Why

Port-reuse pool behavior is now part of scale/recovery logic, but status payload had no direct visibility for reclaimed-port backlog. Operators need this metric to understand reuse health under churn.

## Result

- Cluster status payload now exposes reusable-port pool size in real time.
- Scale diagnostics can distinguish fresh-port growth from healthy reclaimed-port reuse.
- Existing status-snapshot elision behavior remains deterministic.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
