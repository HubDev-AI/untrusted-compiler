# M39: LASM Next-Live Resolver Start-Index Fast Path

Date: 2026-02-21  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `resolve_lasm_cluster_next_live_sender_index(...)` in `/compiler/sec4-cli/src/main.rs`.
- Added immediate return when `start_index_wrapped` already points to a live sender.
- Cache lookup and scan fallback remain unchanged for dead start-index cases.

## Why

Resolver calls frequently start from already-live indices in degraded dispatch paths. Running through lookup/scan plumbing in those cases adds avoidable overhead.

## Result

- Reduced resolver work for already-live start positions.
- Preserved deterministic live-index selection behavior for dead start positions.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
