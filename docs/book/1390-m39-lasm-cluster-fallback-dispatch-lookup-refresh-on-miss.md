# M39: LASM Cluster Fallback-Dispatch Lookup Refresh-On-Miss

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated fallback dispatch in:
  - `/Users/vladimirtrifonov/src/ai/AILang/compiler/sec4-cli/src/lasm_cluster_fallback_dispatch.rs`
  - `/Users/vladimirtrifonov/src/ai/AILang/compiler/sec4-cli/src/lasm_cluster_accept_loop.rs`
- Changes:
  - `dispatch_lasm_cluster_relay_stream_fallback_multi(...)` now receives mutable next-live lookup state,
  - fallback scan/index helpers now refresh next-live lookup table once on lookup miss and retry,
  - accept-loop fallback dispatch wiring now passes mutable lookup state into fallback dispatch helpers.

## Why

Fallback traversal used cached next-live lookup data but, on stale-cache misses, immediately dropped to slower scan defaults without healing cache state. Under topology churn this repeats avoidable miss/scan work. A refresh-and-retry keeps lookup-based traversal hot while preserving deterministic fallback semantics.

## Result

- Fallback traversal self-heals lookup cache on miss.
- Reduced repeated stale-lookup fallback scan overhead in degraded sender topologies.
- Existing saturation/unavailable error behavior remains unchanged.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
