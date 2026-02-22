# M39: LASM Cluster Accept-Loop Dispatch Error Wrapper

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `/Users/vladimirtrifonov/src/ai/AILang/compiler/sec4-cli/src/lasm_cluster_accept_loop.rs`:
  - added `handle_lasm_cluster_accept_dispatch_error_with_counters(...)`,
  - updated accept-loop single-sender and multi-sender dispatch-error call sites to use this wrapper,
  - `handle_lasm_cluster_accept_unavailable_stream(...)` now reuses the same wrapper path for `Unavailable(...)` dispatch errors.

## Why

Accept-loop dispatch-error branches repeated the same counter/atomic argument threading. A single wrapper keeps that wiring centralized and reduces repeated call-site complexity.

## Result

- One counter-wiring path for dispatch-error handling in accept loop.
- Unavailable and generic dispatch-error branches share the same wrapper.
- No change to dispatch-error envelopes, counters, or control flow semantics.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
