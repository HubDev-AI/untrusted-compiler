# M39: LASM Fallback Dispatch Module Extraction

Date: 2026-02-21  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Added module file `$REPO_ROOT/compiler/sec4-cli/src/lasm_cluster_fallback_dispatch.rs`.
- Moved fallback-dispatch internals out of `main.rs`:
  - `dispatch_lasm_cluster_relay_stream_fallback_dual_live(...)`
  - `dispatch_lasm_cluster_relay_stream_fallback_single_live(...)`
  - `dispatch_lasm_cluster_relay_stream_fallback_multi(...)`
  - fallback terminal/scan-advance helper internals
- `main.rs` now imports fallback-dispatch functions from the module.

## Why

Fallback dispatch mechanics were one of the largest remaining hot-path blocks in `main.rs`. Extracting them keeps accept-loop orchestration in the CLI entry file while isolating fallback-specific behavior in a dedicated runtime module.

## Result

- Significant reduction of fallback mechanics in `main.rs`.
- No behavior change in saturated/unavailable fallback handling.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
