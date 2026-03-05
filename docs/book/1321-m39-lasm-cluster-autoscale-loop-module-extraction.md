# M39: LASM Cluster Autoscale Loop Module Extraction

Date: 2026-02-21  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Added module file `$REPO_ROOT/compiler/sec4-cli/src/lasm_cluster_autoscale_loop.rs`.
- Moved the autoscale worker thread loop from `cmd_run_lasm_cluster(...)` in `main.rs` into:
  - `spawn_lasm_cluster_autoscale_loop(...)`
  - `LasmClusterAutoscaleLoopConfig`
- `main.rs` now delegates autoscale thread startup to this module with an explicit config surface.

## Why

Autoscale flow was still a large inline closure in cluster orchestration. Extracting it continues the multi-file LASM decomposition and isolates autoscale lifecycle/tuning mechanics from CLI orchestration glue.

## Result

- Smaller `main.rs` around cluster bootstrap/run/shutdown flow.
- Autoscale behavior preserved:
  - target/boost sizing decisions,
  - cooldown bookkeeping and status counters,
  - worker spawn/downscale lifecycle updates.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
