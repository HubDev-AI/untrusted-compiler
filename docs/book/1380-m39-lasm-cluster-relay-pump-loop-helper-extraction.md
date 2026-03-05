# M39: LASM Cluster Relay Pump Loop Helper Extraction

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `$REPO_ROOT/compiler/sec4-cli/src/lasm_cluster_relay_worker_loop.rs`:
  - extracted relay pump scheduler block into `pump_lasm_cluster_relay_connections(...)`,
  - worker loop now calls this helper and receives a single `progressed` signal,
  - existing per-step helper calls (`pump_lasm_cluster_relay_connection_once`, release/warning/cursor helpers) are reused unchanged inside the extracted loop helper.

## Why

Relay worker loop still embedded a long pump-scheduler block after recent helper work. Extracting that block into one helper reduces outer-loop branch volume and keeps pump scheduling logic single-owned.

## Result

- Outer relay worker loop is shorter and focused on accept/fallback/counter-flush flow.
- Pump scheduling behavior remains unchanged across full-scan and budgeted modes.
- Warning-throttle/release/cursor semantics are preserved.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
