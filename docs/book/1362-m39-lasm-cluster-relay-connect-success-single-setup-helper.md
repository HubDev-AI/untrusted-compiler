# M39: LASM Cluster Relay Connect-Success Single Setup Helper

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `/Users/vladimirtrifonov/src/ai/AILang/compiler/sec4-cli/src/lasm_cluster_relay_worker_loop.rs`:
  - extracted relay connect-success setup into `initialize_lasm_cluster_relay_connection(...)`,
  - both primary and fallback connect-success paths now call this helper for:
    - `TCP_NODELAY` setup,
    - pooled-buffer relay pump creation,
    - deterministic throttled relay-init warning handling,
    - active-connection decrement on init failure.

## Why

Primary and fallback connect-success branches had duplicated relay setup logic. Keeping one setup path removes drift risk while preserving runtime behavior under ongoing hot-path tuning.

## Result

- One connect-success setup path for relay worker loop.
- Lower maintenance risk while preserving deterministic warning/decrement semantics.
- No change to fallback selection order, saturation semantics, or failure envelopes.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
