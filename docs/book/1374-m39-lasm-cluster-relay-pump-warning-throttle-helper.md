# M39: LASM Cluster Relay Pump Warning-Throttle Helper

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `/Users/vladimirtrifonov/src/ai/AILang/compiler/sec4-cli/src/lasm_cluster_relay_worker_loop.rs`:
  - added helper `emit_lasm_cluster_relay_pump_warning_if_allowed(...)`,
  - both relay pump error branches now call this helper instead of duplicating throttle checks,
  - helper centralizes warning window comparison and throttle window advancement.

## Why

Relay pump scheduler had duplicate warning-throttle blocks in both full-scan and budgeted modes. A shared helper keeps warning emission logic centralized and prevents drift in throttling semantics across schedulers.

## Result

- One warning-throttle path for relay pump error logs.
- Reduced duplicated warning branch code in relay worker loop.
- No change to warning message or throttle timing semantics.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
