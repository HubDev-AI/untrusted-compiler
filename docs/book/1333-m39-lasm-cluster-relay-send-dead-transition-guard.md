# M39: LASM Cluster Relay Send Dead-Transition Guard

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `$REPO_ROOT/compiler/sec4-cli/src/lasm_cluster_relay_send.rs`:
  - `attempt_lasm_cluster_relay_send` disconnected path now updates sender-liveness state only on live-to-dead transition,
  - relay live-sender count now decrements with direct bounded subtraction only when that transition occurs,
  - removed unconditional `saturating_sub` decrement on every disconnected send attempt.

## Why

Under repeated disconnected attempts against the same sender, unconditional saturating decrements could repeatedly touch counters despite no state transition. Transition-guarded updates remove that churn and keep live-count tracking tighter.

## Result

- Reduced disconnected-path counter churn.
- Preserved relay disconnect fallback semantics with tighter liveness accounting.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
