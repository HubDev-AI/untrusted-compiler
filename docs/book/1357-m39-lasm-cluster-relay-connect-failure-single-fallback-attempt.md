# M39: LASM Cluster Relay Connect-Failure Single Fallback Attempt

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `/Users/vladimirtrifonov/src/ai/AILang/compiler/sec4-cli/src/lasm_cluster_relay_worker_loop.rs`:
  - on primary selected backend connect failure, relay worker now marks primary backend unhealthy and performs one immediate connect attempt to another currently-healthy backend (if available),
  - if fallback connect succeeds, request proceeds via relay pump path without emitting worker-unavailable response,
  - if fallback connect also fails (or no alternate healthy backend exists), behavior falls back to existing worker-unavailable response path.

## Why

Transient connect failures on one backend should not force immediate request failure when another healthy backend is available in the same relay worker iteration.

## Result

- Improved degraded connect resilience with bounded single-fallback behavior.
- Preserved deterministic unhealthy backend cooldown tracking and warning throttling.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
