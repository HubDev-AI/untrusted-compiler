# M39: LASM Fallback Two-Sender Single-Helper Reuse

Date: 2026-02-21  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `dispatch_lasm_cluster_relay_stream_fallback_multi(...)` in `/compiler/sec4-cli/src/main.rs`.
- Replaced inline two-sender fallback branch body (`sender_count == 2`) with direct reuse of:
  - `dispatch_lasm_cluster_relay_stream_fallback_single_live(...)`.

## Why

The two-sender fallback branch duplicated single-live fallback behavior (live-slot gate, send/full/disconnect handling, and deterministic terminal result). Reusing the existing inlined helper removes duplicate logic without changing semantics.

## Result

- Reduced duplicate code in fallback two-sender dispatch path.
- Preserved deterministic saturated/unavailable behavior and liveness updates.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
