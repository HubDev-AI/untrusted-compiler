# M39: LASM Cluster Accept-Loop Fallback Lookup-State Plumbing

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `/Users/vladimirtrifonov/src/ai/AILang/compiler/sec4-cli/src/lasm_cluster_fallback_dispatch.rs`:
  - added `dispatch_lasm_cluster_relay_stream_fallback_multi_with_lookup_state(...)`,
  - this helper selects lookup slice once from a precomputed boolean and forwards to existing fallback-multi dispatcher.
- Updated `/Users/vladimirtrifonov/src/ai/AILang/compiler/sec4-cli/src/lasm_cluster_accept_loop.rs`:
  - fallback path now computes `relay_use_next_live_lookup_for_fallback` once per primary-dispatch failure,
  - single-live/dual-live/default fallback branches now reuse the same helper and lookup-state flag.

## Why

Primary dispatch failure handling had repeated lookup-slice plumbing across fallback branches. This slice makes lookup-state handling explicit and shared per failure event, trimming branch/argument duplication in a hot path.

## Result

- Leaner accept-loop fallback branch plumbing.
- No behavior change in fallback dispatch semantics.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
