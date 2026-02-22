# M39: LASM Cluster Worker Port Reuse Pool

Date: 2026-02-22  
Milestone: M39 (scaling/runtime hot-path tuning)

## What Changed

- Added reusable worker-port pool in LASM cluster state.
- Updated port reservation/reclaim paths:
  - min-worker recovery reservation now consumes from `reusable_ports` first, then `next_port`
  - dead-worker pruning now reclaims removed worker ports into the pool
  - autoscale scale-down now reclaims removed worker ports into the pool

## Why

Without a reuse pool, repeated scale-up/scale-down/dead-worker cycles only consume new ports from `next_port`, increasing risk of avoidable port-range churn in long-running clusters.

## Result

- Scale cycles can reuse reclaimed ports deterministically.
- Port allocation pressure on `next_port` is reduced under churn.
- Existing autoscale/dead-worker semantics remain unchanged.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
