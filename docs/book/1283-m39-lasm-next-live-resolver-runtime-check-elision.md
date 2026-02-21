# M39: LASM Next-Live Resolver Runtime Check Elision

Date: 2026-02-21  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `resolve_lasm_cluster_next_live_sender_index(...)` in `/compiler/sec4-cli/src/main.rs`.
- Replaced runtime cache-shape check (`lookup.len() == relay_sender_live.len()`) with debug assertion:
  - `debug_assert_eq!(lookup.len(), relay_sender_live.len())`
- Kept runtime branch only for cache-presence (`!lookup.is_empty()`) and existing fallback-to-scan behavior.

## Why

Cache shape invariants are controlled by accept-loop setup and usage boundaries. Rechecking cache length on each resolver call adds avoidable branch work in degraded cached lookup paths.

## Result

- Slightly reduced branch work in hot resolver calls.
- No behavior change in release builds for valid cache paths; absent-cache paths still resolve via live scan.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
