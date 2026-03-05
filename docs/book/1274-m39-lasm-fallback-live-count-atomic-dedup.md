# M39: LASM Fallback Live-Count Atomic Dedup

Date: 2026-02-21  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated fallback disconnect handling in `/compiler/sec4-cli/src/main.rs`.
- Removed immediate `relay_live_sender_count_observed.fetch_min(...)` on primary `TrySendError::Disconnected` in accept-loop fallback handling.
- Kept end-of-fallback live-count delta check and telemetry update path unchanged.

## Why

The fallback branch already has a centralized post-fallback live-count delta check that refreshes hints and updates live-count telemetry when sender liveness changes. Keeping an additional immediate atomic update in the primary disconnect branch caused duplicate writes.

## Result

- One less atomic update on primary-disconnect fallback cycles.
- Telemetry semantics remain the same because live-count transitions are still published through the post-fallback delta branch.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
