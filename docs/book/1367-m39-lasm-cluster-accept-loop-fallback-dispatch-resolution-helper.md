# M39: LASM Cluster Accept-Loop Fallback Dispatch Resolution Helper

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `/Users/vladimirtrifonov/src/ai/AILang/compiler/sec4-cli/src/lasm_cluster_accept_loop.rs`:
  - added `dispatch_lasm_cluster_relay_stream_fallback_with_live_hints(...)`,
  - moved fallback dispatch strategy resolution (all-live / dual-live / single-live / multi) out of the main accept-loop body into this helper.

## Why

Fallback strategy resolution was deeply nested in the accept loop and duplicated fallback-multi calls across branches. Centralizing this logic keeps the hot path simpler and reduces drift risk during further runtime tuning.

## Result

- Main accept loop now calls one fallback-dispatch helper.
- Live-hint refresh behavior remains deterministic and branch-equivalent.
- No change to fallback dispatch error handling or saturation envelopes.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
