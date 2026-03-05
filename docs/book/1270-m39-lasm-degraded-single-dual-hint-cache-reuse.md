# M39: LASM Degraded Single/Dual Hint Cache Reuse

Date: 2026-02-21  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated degraded-path accept dispatch in `/compiler/sec4-cli/src/main.rs`.
- For `relay_live_sender_count == 1` and `relay_live_sender_count == 2`, dispatch now:
  - reuses cached live-hint values (`relay_single_live_sender_index`, `relay_dual_live_sender_indices`) when present,
  - refreshes hints only when cache is missing.

## Why

In degraded steady state, the previous flow rescanned `relay_sender_live` on every request even when live-shard topology had not changed. That added avoidable overhead to an already stressed path.

## Result

- Removes repeated per-request hint scans in single-live and dual-live degraded steady state.
- Keeps deterministic fallback behavior unchanged: if cache is absent, hints are refreshed; if refresh still fails, request gets deterministic unavailable handling.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
