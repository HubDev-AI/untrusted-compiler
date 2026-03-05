# M39: LASM Cluster Shutdown Lock Poison Handling

Date: 2026-02-21  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `$REPO_ROOT/compiler/sec4-cli/src/lasm_cluster_shutdown.rs`:
  - `LasmClusterShutdownSummary` now includes `state_lock_poisoned`,
  - shutdown summary now exposes unified failure check (`has_failures`) and deterministic failure message.
- Updated `$REPO_ROOT/compiler/sec4-cli/src/main.rs`:
  - cluster run now treats shutdown lock-poison as deterministic runtime failure,
  - shutdown failure reporting now uses unified shutdown failure summary.

## Why

If the cluster state lock is poisoned during shutdown, worker-stop cleanup can fail silently. This slice makes that state explicit and non-zero, matching deterministic failure behavior already used for thread-panic shutdown failures.

## Result

- Shutdown lock-poison is now visible and fails LASM cluster run deterministically.
- Shutdown diagnostics now cover both thread panics and poisoned-state cleanup failures.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
