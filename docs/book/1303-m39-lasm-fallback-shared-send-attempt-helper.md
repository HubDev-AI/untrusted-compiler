# M39: LASM Fallback Shared Send-Attempt Helper

Date: 2026-02-21  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Added `attempt_lasm_cluster_relay_send(...)` in `/Users/vladimirtrifonov/src/ai/AILang/compiler/sec4-cli/src/main.rs`.
- Reused this helper in fallback-multi dispatch paths:
  - all-live scan loop,
  - `scan_live_target == 2` first attempt,
  - degraded general scan attempt branch.

## Why

Hot-path fallback code repeated the same `try_send` full/disconnect handling in several branches. Centralizing send-attempt handling reduces duplicate logic while keeping deterministic liveness transitions.

## Result

- Shared full/disconnect send-attempt behavior in fallback multi path.
- Preserved degraded dynamic target updates by updating `scan_live_target_dynamic` only when live sender count changes.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
