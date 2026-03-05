# M39: LASM Cluster Thread Panic Handling

Date: 2026-02-21  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `$REPO_ROOT/compiler/sec4-cli/src/lasm_cluster_accept_workers.rs`:
  - added panic-aware accept worker join helper,
  - accept worker orchestration now returns deterministic `Err(...)` when worker joins report panics.
- Updated `$REPO_ROOT/compiler/sec4-cli/src/lasm_cluster_shutdown.rs`:
  - `finalize_lasm_cluster_runtime(...)` now returns `LasmClusterShutdownSummary` instead of `()`,
  - summary includes panic flags/count for relay workers, autoscale thread, and status-writer thread.
- Updated `$REPO_ROOT/compiler/sec4-cli/src/main.rs`:
  - cluster runtime now reports shutdown panic summary deterministically,
  - successful run path now fails with non-zero exit when shutdown detects background thread panic(s).

## Why

Previously, join failures were ignored in accept worker and shared shutdown flows. That allowed runtime thread panics to pass silently and report success. This slice makes those failure modes observable and deterministic.

## Result

- LASM cluster runtime now surfaces background worker-thread panic conditions explicitly.
- Thread panic conditions produce deterministic runtime failure diagnostics instead of silent success.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
