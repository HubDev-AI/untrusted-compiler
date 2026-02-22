# M39: LASM Cluster Relay Pump Cursor Helper Unification

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `/Users/vladimirtrifonov/src/ai/AILang/compiler/sec4-cli/src/lasm_cluster_relay_worker_loop.rs`:
  - added `advance_lasm_cluster_relay_pump_cursor(...)` for wrapped cursor progression,
  - added `normalize_lasm_cluster_relay_pump_cursor(...)` for post-release empty/bounds normalization,
  - budgeted relay pump scheduler now uses these helpers in `Progressed`, `Idle`, `Complete`, and pump-error branches.

## Why

Cursor progression and normalization logic was duplicated across multiple relay pump branches, making future hot-path changes error-prone. Shared helpers keep cursor semantics centralized without changing scheduling behavior.

## Result

- One wrapped cursor progression path.
- One cursor normalization path after relay release.
- No change to deterministic cursor wrap/break behavior in the budgeted pump scheduler.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
