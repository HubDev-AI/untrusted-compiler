# M39: LASM Cluster Relay Connect Fallback Next-Healthy Cyclic Scan

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `/Users/vladimirtrifonov/src/ai/AILang/compiler/sec4-cli/src/lasm_cluster_relay_worker_loop.rs`:
  - connect-failure fallback backend selection now performs a cyclic scan starting from the failed backend’s next index,
  - removed first-healthy global pick behavior for fallback connect attempts.

## Why

Always selecting the first healthy backend for fallback can create deterministic fallback hot spots under repeated connect failures. A next-healthy cyclic scan keeps fallback distribution aligned with backend order around the failed index.

## Result

- Lower fallback selection bias under repeated connect faults.
- Deterministic fallback candidate ordering preserved.
- Existing unhealthy cooldown/warning behavior unchanged.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
