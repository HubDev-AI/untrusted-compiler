# M39: LASM Cluster Fallback Multi Direct Lookup-State Signature

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `/Users/vladimirtrifonov/src/ai/AILang/compiler/sec4-cli/src/lasm_cluster_fallback_dispatch.rs`:
  - removed `dispatch_lasm_cluster_relay_stream_fallback_multi_with_lookup_state(...)`,
  - `dispatch_lasm_cluster_relay_stream_fallback_multi(...)` now accepts precomputed `relay_has_next_live_sender_lookup` as a direct parameter.
- Updated `/Users/vladimirtrifonov/src/ai/AILang/compiler/sec4-cli/src/lasm_cluster_accept_loop.rs`:
  - fallback call sites now call core fallback-multi function directly with the precomputed lookup-state flag.

## Why

The previous wrapper introduced extra call indirection and repeated lookup-availability branching. Threading lookup-state directly into the core function keeps behavior unchanged while reducing fallback dispatch plumbing overhead.

## Result

- One less function layer in fallback-multi call path.
- One less per-call lookup-availability branch in fallback-multi setup.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
