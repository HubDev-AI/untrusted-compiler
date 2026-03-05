# M39: LASM Cluster Relay Pump Local Cursor Writeback

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `$REPO_ROOT/compiler/sec4-cli/src/lasm_cluster_relay_worker_loop.rs`:
  - relay pump scheduler now runs per-step cursor progression/normalization on a local cursor variable,
  - mode resolver now returns an initial cursor and pump budget (`LasmClusterRelayPumpModeResolution`),
  - shared cursor state (`relay_pump_cursor`) is written once after loop completion (or reset to zero for full-scan mode).

## Why

Per-step updates against shared cursor state were no longer necessary after shared pump-loop extraction. Local cursor progression keeps semantics stable while reducing mutable shared-state writes in the hot path.

## Result

- Single write-back to `relay_pump_cursor` per pump cycle in budget mode.
- Full-scan mode continues to reset cursor to zero.
- Deterministic cursor/remove progression behavior remains unchanged.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
