# M39: LASM Cluster Accept Workers Module Extraction

Date: 2026-02-21  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Added module file `/Users/vladimirtrifonov/src/ai/AILang/compiler/sec4-cli/src/lasm_cluster_accept_workers.rs`.
- Moved accept-worker orchestration from `cmd_run_lasm_cluster(...)` in `main.rs` into:
  - `run_lasm_cluster_accept_workers(...)`
  - `LasmClusterAcceptWorkersConfig`
- The module now owns:
  - listener clone handling for background accept workers,
  - accept worker thread spawn/join,
  - primary accept-loop run on worker index `0`,
  - deterministic stop-flag handling on accept-loop errors.

## Why

Accept-worker orchestration was still embedded in cluster orchestration flow, adding volume and repeated wiring in `main.rs`. Extracting this flow improves module boundaries and keeps the cluster entry function focused on high-level lifecycle orchestration.

## Result

- Smaller and clearer `main.rs` cluster run flow.
- Behavior preserved for:
  - accept-loop error reporting,
  - accept-worker stop/shutdown behavior,
  - listener clone failure propagation.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
