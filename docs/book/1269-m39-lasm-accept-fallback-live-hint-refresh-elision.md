# M39: LASM Accept Fallback Live-Hint Refresh Elision

Date: 2026-02-21  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated LASM cluster accept-loop fallback handling in `/compiler/sec4-cli/src/main.rs` so live-sender hint refresh and live-count telemetry updates are conditional.
- Added a `live_count_before_fallback` baseline in the fallback error branch.
- Kept immediate live-hint refresh on primary `TrySendError::Disconnected` (required for dual-live fallback selection).
- After fallback dispatch returns, now refreshes:
  - `relay_single_live_sender_index` / `relay_dual_live_sender_indices`, and
  - `relay_live_sender_count_observed.fetch_min(...)`
  only when fallback actually changed `relay_live_sender_count`.

## Why

Under saturation-heavy paths where fallback returns `Saturated` without disconnecting shards, the previous flow still re-scanned live-sender hints and wrote relay-live telemetry atomics every time. That added avoidable overhead in the proxy hot path.

## Result

- Removes redundant hint refresh scans and atomic writes on saturated-but-live fallback paths.
- Preserves degraded-mode correctness when disconnections do happen (including dual-live fallback behavior).
- Keeps deterministic error and liveness semantics unchanged.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
