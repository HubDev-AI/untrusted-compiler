# M39: LASM Fallback Dual Second-Live Fast Path

Date: 2026-02-21  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `dispatch_lasm_cluster_relay_stream_fallback_multi(...)` in `/Users/vladimirtrifonov/src/ai/AILang/compiler/sec4-cli/src/main.rs`.
- In the `scan_live_target == 2` branch:
  - added direct wrapped-next liveness check for the second attempt (`second_start_index`) before resolver fallback,
  - routed second-attempt dispatch through `dispatch_lasm_cluster_relay_stream_fallback_single_live(...)`.

## Why

When the wrapped next slot is already live, second-attempt resolver work is unnecessary. Reusing the single-live helper also removes duplicate second-attempt send/disconnect logic.

## Result

- Faster second-attempt path for adjacent-live dual fallback states.
- Preserved deterministic saturated/unavailable behavior and liveness transitions.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
