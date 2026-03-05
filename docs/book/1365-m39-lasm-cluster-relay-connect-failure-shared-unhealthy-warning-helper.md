# M39: LASM Cluster Relay Connect-Failure Shared Unhealthy/Warning Helper

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `$REPO_ROOT/compiler/sec4-cli/src/lasm_cluster_relay_worker_loop.rs`:
  - added shared helper `mark_lasm_cluster_relay_backend_connect_failure(...)`,
  - both primary connect-failure and fallback connect-failure branches now call this helper for:
    - unhealthy cooldown marking,
    - unhealthy prune scheduling,
    - connect warning throttling,
    - selection-lookup dirty signaling.

## Why

Failure bookkeeping was duplicated across primary/fallback branches and easy to drift during ongoing runtime tuning. One helper keeps failure semantics aligned and keeps the hot path easier to evolve safely.

## Result

- One deterministic connect-failure bookkeeping path.
- Reduced relay-worker-loop duplication.
- No change to fallback routing order, saturation accounting, or response envelopes.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
