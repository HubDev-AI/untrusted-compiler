# M39: LASM Cluster Accept-Loop Redundant Live Guard Removal

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `$REPO_ROOT/compiler/sec4-cli/src/lasm_cluster_accept_loop.rs`:
  - removed redundant `relay_live_sender_count > 0` guard from dispatch-cursor realignment condition,
  - retained `relay_live_sender_count > 1` guard, which already subsumes the removed check.

## Why

This condition is evaluated on a hot fallback-completion path. Eliminating redundant conjunction checks reduces branch work without changing behavior.

## Result

- Slightly leaner accept-loop realignment condition.
- No behavior change in dispatch-cursor realignment semantics.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
