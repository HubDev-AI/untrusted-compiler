# M39: LASM Cluster Relay Fallback Alternate-Only Connect Attempts

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `$REPO_ROOT/compiler/sec4-cli/src/lasm_cluster_relay_worker_loop.rs`:
  - fallback connect traversal now excludes the primary-selected backend that just failed connect,
  - identity traversal now scans exactly `worker_count - 1` alternates,
  - healthy-lookup traversal now defensively skips the selected backend index if present.

## Why

Retrying the same failed backend inside fallback adds unnecessary latency and connect pressure while not improving recovery odds. Fallback should attempt only alternates.

## Result

- Fallback attempts are strictly alternate-backend attempts.
- Lower redundant connect retries under primary connect-failure paths.
- No change to deterministic warning/cooldown/saturation contracts.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
