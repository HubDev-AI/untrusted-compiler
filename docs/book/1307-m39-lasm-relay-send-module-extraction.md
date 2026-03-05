# M39: LASM Relay Send Module Extraction

Date: 2026-02-21  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Added module file `$REPO_ROOT/compiler/sec4-cli/src/lasm_cluster_relay_send.rs`.
- Moved relay send helpers out of `main.rs`:
  - `attempt_lasm_cluster_relay_send(...)`
  - `attempt_lasm_cluster_relay_send_single(...)`
- Wired `main.rs` imports through the new module.

## Why

These helpers were part of LASM cluster dispatch internals but still lived in the monolithic `main.rs`. Extracting them reduces file concentration and advances the multi-file runtime decomposition priority.

## Result

- Cleaner module boundary for relay send-attempt logic.
- No behavioral/semantic change in LASM cluster dispatch flow.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
