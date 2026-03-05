# M39: LASM Cluster Relay Pump Release Helper

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `$REPO_ROOT/compiler/sec4-cli/src/lasm_cluster_relay_worker_loop.rs`:
  - added helper `release_lasm_cluster_relay_connection(...)`,
  - relay pump `Complete` and pump-error branches now call this helper instead of repeating inline release code,
  - helper centralizes `swap_remove`, pooled-buffer return, and active-connection decrement bookkeeping.

## Why

Relay pump loop had duplicated release logic in multiple branches across full-scan and budgeted pump modes. A shared helper keeps release behavior centralized so future hot-path changes cannot drift across branch copies.

## Result

- One release path for relay pump completion/error cleanup.
- Reduced duplicated branch-body code in relay worker loop.
- No change to active-decrement, buffer reuse, or cursor flow semantics.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
