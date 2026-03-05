# M39: LASM Cluster Relay Pump Shared Loop

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `$REPO_ROOT/compiler/sec4-cli/src/lasm_cluster_relay_worker_loop.rs`:
  - collapsed full-scan and budgeted relay pump scheduler branches into one shared loop,
  - added mode flag `full_scan_pump_mode` to choose between full-scan and budgeted pump budget sizing,
  - preserved full-scan cursor reset behavior before/after the shared loop.

## Why

After step/release/warning/cursor helper extraction, the remaining scheduler split duplicated pump-loop scaffolding. One shared loop removes this duplication while keeping mode-specific semantics explicit.

## Result

- One relay pump scheduling loop for both full-scan and budgeted modes.
- Deterministic remove/cursor progression behavior remains unchanged.
- Full-scan mode still resets pump cursor to zero.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
