# M39: LASM Cluster Accept Error Propagation

Date: 2026-02-21  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `/Users/vladimirtrifonov/src/ai/AILang/compiler/sec4-cli/src/lasm_cluster_accept_workers.rs`:
  - added deterministic first-error capture for accept-loop failures across worker/main accept paths,
  - removed inline error printing from worker orchestration,
  - now returns non-`Ok` result when any accept-loop failure occurs (instead of returning success after logging).
- `cmd_run_lasm_cluster(...)` now receives propagated accept-loop errors and exits non-zero while preserving shared shutdown finalization behavior.

## Why

Previously, accept-loop failures could be logged while cluster run still returned success. That masked runtime failures and made operator behavior non-deterministic. This slice makes accept-loop failure a hard runtime failure.

## Result

- Any accept-loop failure now deterministically fails LASM cluster run.
- Error reporting is centralized through caller-facing runtime error handling.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
