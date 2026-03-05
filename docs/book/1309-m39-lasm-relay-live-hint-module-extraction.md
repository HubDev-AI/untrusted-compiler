# M39: LASM Relay Live-Hint Module Extraction

Date: 2026-02-21  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `$REPO_ROOT/compiler/sec4-cli/src/lasm_cluster_relay_topology.rs`.
- Moved live-hint refresh helpers from `main.rs` into the topology module:
  - `refresh_lasm_cluster_next_live_sender_lookup(...)`
  - `refresh_lasm_cluster_single_live_sender_index(...)`
  - `refresh_lasm_cluster_dual_live_sender_indices(...)`
  - `refresh_lasm_cluster_live_sender_hints(...)`
- `main.rs` now imports these helpers from module boundaries.

## Why

These hint-refresh helpers are relay topology responsibilities and were still embedded in `main.rs`. Extracting them reduces monolithic CLI surface and keeps topology logic together.

## Result

- Cleaner module ownership for relay topology and live-hint maintenance.
- No behavioral changes in accept-loop/fallback dispatch paths.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
