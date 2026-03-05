# M39: LASM Cluster Relay Pump Step Shared Helper

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `$REPO_ROOT/compiler/sec4-cli/src/lasm_cluster_relay_worker_loop.rs`:
  - added `pump_lasm_cluster_relay_connection_once(...)` and `LasmClusterRelayPumpDispatchOutcome`,
  - full-scan and budgeted pump schedulers now call the same helper for `pump_once` outcomes,
  - helper centralizes release handling and warning-throttle/decrement side effects for `Complete` and error paths.

## Why

Both pump schedulers duplicated the same `pump_once` outcome branches. One shared helper keeps branch behavior aligned across schedulers and reduces repeated hot-path code.

## Result

- One `pump_once` outcome path shared by both relay pump schedulers.
- Reduced duplicated branch bodies for progressed/idle/release/error handling.
- No change to warning-throttle, release, or progression semantics.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
