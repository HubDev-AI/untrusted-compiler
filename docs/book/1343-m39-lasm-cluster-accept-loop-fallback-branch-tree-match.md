# M39: LASM Cluster Accept-Loop Fallback Branch Tree Match

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `$REPO_ROOT/compiler/sec4-cli/src/lasm_cluster_accept_loop.rs`:
  - failed-primary-dispatch fallback selection now checks `relay_all_senders_live` first,
  - degraded path selection now uses `match relay_live_sender_count` (`2`, `1`, default) instead of repeated boolean conjunction checks (`!relay_all_senders_live && count == ...`).

## Why

Fallback branch selection is in the accept-loop hot path when primary dispatch misses. Consolidating branch structure reduces repeated boolean conjunction evaluation while keeping fallback behavior intact.

## Result

- Leaner branch tree in accept-loop fallback dispatch selection.
- No behavior change in dual/single/default fallback dispatch outcomes.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
