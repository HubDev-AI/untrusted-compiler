# M39: LASM Cluster Relay Fallback Traversal Mode Split with Shared Attempt Helper

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `$REPO_ROOT/compiler/sec4-cli/src/lasm_cluster_relay_worker_loop.rs`:
  - added shared helper `try_lasm_cluster_relay_fallback_connect(...)` for one fallback connect attempt,
  - fallback traversal now uses explicit mode loops:
    - single-healthy candidate mode,
    - identity-order traversal mode,
    - healthy-lookup traversal mode.

## Why

Mixed traversal modes in one per-iteration branch path made fallback flow harder to optimize and reason about. Splitting traversal modes while sharing the attempt body keeps failure-path logic clearer and reduces per-iteration branching.

## Result

- Clear traversal-mode separation in fallback connect path.
- One shared fallback attempt body for connect success/failure handling.
- No change to deterministic warning/cooldown/saturation contracts.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
