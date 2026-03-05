# M39: LASM Single-Sender Dispatch Helper

Date: 2026-02-21  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `$REPO_ROOT/compiler/sec4-cli/src/main.rs`.
- Added `attempt_lasm_cluster_relay_send_single(...)` for single-sender relay dispatch.
- Single-sender accept loop now delegates dispatch result mapping through this helper before `handle_lasm_cluster_accept_dispatch_error(...)`.

## Why

Single-sender accept path still had local `try_send` to dispatch-error mapping even after shared helper work in multi-sender paths. Extracting a dedicated single-sender helper centralizes this mapping and keeps accept dispatch logic consistent.

## Result

- Reduced duplicate mapping code in single-sender accept path.
- Preserved deterministic saturated/unavailable error routing for single-sender mode.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
