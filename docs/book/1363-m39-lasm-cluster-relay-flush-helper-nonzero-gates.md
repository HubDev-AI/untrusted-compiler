# M39: LASM Cluster Relay Flush-Helper Non-Zero Gates

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `$REPO_ROOT/compiler/sec4-cli/src/lasm_cluster_relay_worker_loop.rs`:
  - saturation flush helper is now called only when local saturation counters are non-zero,
  - active-connection decrement flush helper is now called only when local active-decrement counter is non-zero.

## Why

Relay loop cycles frequently in steady-state and idle paths. Avoiding helper calls when there is no pending counter work reduces per-cycle overhead in the hot loop.

## Result

- Lower no-op helper-call overhead in relay worker cycles.
- No change to counter semantics or atomic flush behavior when pending deltas exist.
- Existing saturation and active-connection accounting remains deterministic.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
