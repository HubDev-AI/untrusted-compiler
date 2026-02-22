# M39: LASM Relay Batch Snapshot Clone Elision

Date: 2026-02-22  
Milestone: M39 (scaling/runtime hot-path tuning)

## What Changed

- Updated LASM relay worker loop batch handling to track whether worker-port snapshot loading is already done for the current batch.
- Kept changed-topology behavior intact (`ArcSwap` load + remap path) while skipping snapshot `Arc` cloning when topology is unchanged.
- Reused selected snapshot directly on unchanged batches.

## Why

The relay worker accepted-request path cloned the selected worker-port snapshot into a temporary option even when topology was unchanged. Under steady traffic this created avoidable per-batch `Arc` refcount churn on a hot path.

## Result

- Unchanged topology no longer pays per-batch snapshot clone overhead.
- Topology-change remap behavior remains deterministic and unchanged.
- Relay selection/fallback logic remains functionally equivalent.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
