# M39: LASM Bulk Reuse Port Reservation

Date: 2026-02-22  
Milestone: M39 (scaling/runtime hot-path tuning)

## What Changed

- Updated `reserve_lasm_cluster_worker_ports(...)` to reserve reclaimed ports in bulk.
- Reusable ports are now appended with one slice copy (`extend_from_slice`) and removed with one `truncate`.
- Fresh port allocation remains deterministic via `next_port` for the remaining reservation tail.

## Why

Per-port `pop` loops added avoidable branch churn during scale-up/recovery reservation planning. Since `reusable_ports` is a contiguous vector, reservation can consume tail ranges in bulk.

## Result

- Lower reservation-loop overhead on scale/recovery planning paths.
- Reclaimed-port preference behavior remains unchanged.
- Deterministic port allocation semantics are preserved.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
