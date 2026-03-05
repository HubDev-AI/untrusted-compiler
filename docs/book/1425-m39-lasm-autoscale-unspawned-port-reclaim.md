# M39: LASM Autoscale Unspawned Port Reclaim

Date: 2026-02-22  
Milestone: M39 (scaling/runtime hot-path tuning)

## What Changed

- Updated LASM autoscale scale-up apply path to track partial spawn progress.
- When spawn fails mid-batch, unspawned reserved ports are now reclaimed into `reusable_ports`.
- Existing spawned workers are still applied as before, and the failure warning path remains unchanged.

## Why

After adding reusable port pooling, spawn-failure batches could still lose reserved ports that were not yet spawned. Reclaiming those reserved ports keeps port reuse deterministic and prevents avoidable port churn.

## Result

- Reserved ports are no longer dropped on mid-batch spawn failure.
- Subsequent recovery/scale-up cycles can reuse the reclaimed ports.
- Autoscale behavior remains deterministic under partial-failure conditions.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
