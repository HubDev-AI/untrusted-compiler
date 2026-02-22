# M39: LASM Cluster Relay Topology Direct Start-Index Check

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `/Users/vladimirtrifonov/src/ai/AILang/compiler/sec4-cli/src/lasm_cluster_relay_topology.rs`:
  - `resolve_lasm_cluster_next_live_sender_index` now exits early on empty sender slices,
  - start-index live-state check now uses direct index access (`relay_sender_live[start_index_wrapped]`) guarded by explicit debug assertion,
  - removed option-chain access (`get(...).copied().unwrap_or(...)`) from this helper.

## Why

This helper is called by degraded relay dispatch paths for next-live sender resolution. Direct index checks reduce option-chain overhead while preserving the same topology resolution behavior.

## Result

- Lower per-call overhead in shared relay topology helper.
- No behavior change for next-live sender resolution.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
