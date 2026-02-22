# M39: LASM Cluster Relay Pump Mode Resolution Helper

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `/Users/vladimirtrifonov/src/ai/AILang/compiler/sec4-cli/src/lasm_cluster_relay_worker_loop.rs`:
  - added `resolve_lasm_cluster_relay_pump_mode(...)` and `LasmClusterRelayPumpModeResolution`,
  - relay pump scheduler now delegates full-scan/budget mode setup to this helper (full-scan cursor reset, budgeted cursor normalization, pump-budget resolution),
  - shared pump loop helper consumes the mode resolution result.

## Why

Relay pump mode setup remained embedded inside scheduler logic. A dedicated resolver keeps scheduler mode arithmetic and cursor setup single-sourced and easier to evolve during M39 tuning.

## Result

- One helper owns relay pump mode selection and budget setup.
- Relay pump scheduler code path is shorter and less branch-heavy.
- No change to deterministic full-scan vs budgeted scheduling behavior.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
