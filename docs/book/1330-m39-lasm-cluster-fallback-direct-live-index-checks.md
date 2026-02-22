# M39: LASM Cluster Fallback Direct Live-Index Checks

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `/Users/vladimirtrifonov/src/ai/AILang/compiler/sec4-cli/src/lasm_cluster_fallback_dispatch.rs`:
  - `dispatch_lasm_cluster_relay_stream_fallback_single_live` and `dispatch_lasm_cluster_relay_stream_fallback_dual_live` now use direct indexed reads for live-sender membership checks,
  - removed `get(...).copied().unwrap_or(...)` option-chain checks from these fast paths,
  - added explicit debug assertions for relay-live vector shape parity (`relay_sender_live.len() == relay_senders.len()`).

## Why

These fallback branches are on the relay dispatch hot path when preferred dispatch misses. Direct index checks remove option-chain overhead while keeping deterministic fallback behavior unchanged.

## Result

- Reduced per-request overhead in single/dual-live fallback checks.
- No change to fallback terminal outcome semantics.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
