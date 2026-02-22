# M39: LASM Cluster Resolve Next-Live With Lookup State

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `/Users/vladimirtrifonov/src/ai/AILang/compiler/sec4-cli/src/lasm_cluster_relay_topology.rs`:
  - added `resolve_lasm_cluster_next_live_sender_index_with_lookup_state(...)`,
  - this helper accepts precomputed lookup availability and avoids recomputing `lookup.is_empty()` in hot callers.
- Updated `/Users/vladimirtrifonov/src/ai/AILang/compiler/sec4-cli/src/lasm_cluster_fallback_dispatch.rs`:
  - degraded fallback path now computes `relay_has_next_live_sender_lookup` once,
  - all next-live resolver calls in that path use the lookup-state-aware helper.

## Why

Fallback degraded dispatch repeatedly resolved next-live sender indices and re-checked lookup availability on each call. Hoisting lookup availability and threading it through resolver calls reduces repeated branch work in this hot path.

## Result

- Lower next-live resolver branch overhead in degraded fallback dispatch.
- No behavior change in degraded fallback sender selection semantics.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
