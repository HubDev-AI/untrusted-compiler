# M39: LASM Fallback Terminal Result Helper

Date: 2026-02-21  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Added inline helper `lasm_cluster_fallback_terminal_dispatch_error(...)` in `/compiler/sec4-cli/src/main.rs`.
- Replaced duplicated terminal fallback return branches in:
  - `dispatch_lasm_cluster_relay_stream_fallback_dual_live(...)`,
  - `dispatch_lasm_cluster_relay_stream_fallback_single_live(...)`,
  - `dispatch_lasm_cluster_relay_stream_fallback_multi(...)`.

## Why

Fallback dispatch paths repeated the same terminal `saturated` vs `unavailable` branch logic across multiple branches/functions. Centralizing this logic keeps hot-path behavior consistent and reduces duplicate branch code.

## Result

- Shared deterministic terminal-result behavior across fallback paths.
- Smaller fallback branch surface with unchanged runtime semantics.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
