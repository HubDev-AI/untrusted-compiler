# M39: LASM Cluster Fallback Scan Lookup-Presence Hoist

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `/Users/vladimirtrifonov/src/ai/AILang/compiler/sec4-cli/src/lasm_cluster_fallback_dispatch.rs`:
  - `dispatch_lasm_cluster_relay_stream_fallback_multi` now computes `relay_has_next_live_sender_lookup` once per dispatch call,
  - scan-advance helper calls now receive that precomputed flag,
  - `advance_lasm_cluster_fallback_scan_index` no longer checks `relay_next_live_sender_lookup.is_empty()` on every invocation.

## Why

Fallback scan-advance executes in a hot relay path when preferred dispatch fails. Hoisting lookup-presence detection removes repeated per-step emptiness checks while retaining the same live-sender resolution behavior.

## Result

- Reduced branch work in repeated fallback scan-advance steps.
- No behavior change in fallback dispatch/live-sender resolution semantics.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
