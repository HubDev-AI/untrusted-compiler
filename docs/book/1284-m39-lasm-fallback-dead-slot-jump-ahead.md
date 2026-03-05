# M39: LASM Fallback Dead-Slot Jump Ahead

Date: 2026-02-21  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `dispatch_lasm_cluster_relay_stream_fallback_multi(...)` in `/compiler/sec4-cli/src/main.rs`.
- In the general degraded scan branch (`scan_live_target > 2`), dead-slot handling now:
  - computes wrapped next start index,
  - resolves next live candidate via `resolve_lasm_cluster_next_live_sender_index(...)`,
  - jumps directly to that candidate when available (or wrapped next start as fallback).

## Why

The previous loop advanced one slot at a time through dead indices in degraded sparse-live pools. That repeated dead-slot checks and delayed reaching live candidates.

## Result

- Fewer dead-slot iterations in degraded fallback scans.
- Preserved deterministic fallback bounds and saturated/unavailable behavior.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
