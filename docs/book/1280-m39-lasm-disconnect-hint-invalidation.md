# M39: LASM Disconnect Hint Invalidation

Date: 2026-02-21  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated primary `TrySendError::Disconnected` handling in `run_lasm_cluster_accept_loop(...)` (`/compiler/sec4-cli/src/main.rs`).
- Replaced immediate `refresh_lasm_cluster_live_sender_hints(...)` call with direct hint invalidation:
  - `relay_single_live_sender_index = None`
  - `relay_dual_live_sender_indices = None`

## Why

Disconnect path previously performed immediate hint refresh and then could refresh hints again in the existing post-fallback live-count-change branch. The immediate refresh is unnecessary because fallback selection already has on-demand hint refresh guards when cached hints are absent.

## Result

- Removed duplicate disconnect-path hint refresh work.
- Kept deterministic fallback behavior unchanged because hint consumers still refresh when hints are absent.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
