# M39: LASM Autoscale Scale-Up Reuse Port Pool

Date: 2026-02-22  
Milestone: M39 (scaling/runtime hot-path tuning)

## What Changed

- Added shared lifecycle helper `reserve_lasm_cluster_worker_ports(...)`.
- Updated min-worker recovery reservation to use the shared helper.
- Updated autoscale scale-up reservation path to use the same helper instead of allocating only from `next_port`.

## Why

Reusable worker ports were already reclaimed on scale-down/dead-worker prune, but autoscale scale-up still consumed only new `next_port` values. That left reclaim benefits partially unused and caused avoidable port churn during repeated scale cycles.

## Result

- Autoscale scale-up now reuses reclaimed worker ports before allocating new ones.
- Recovery and scale-up reservation semantics are now centralized in one path.
- Cluster port reuse behavior is deterministic across both recovery and scaling flows.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
