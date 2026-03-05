# M39: LASM Accept Primary Shared Send Helper

Date: 2026-02-21  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `$REPO_ROOT/compiler/sec4-cli/src/main.rs`.
- In multi-sender accept-loop primary dispatch:
  - replaced direct `try_send` match with `attempt_lasm_cluster_relay_send(...)`,
  - kept existing saturated short-circuit branch (`listener_all_senders_saturated_in_batch`) and fallback dispatch flow.

## Why

Primary accept dispatch was still maintaining its own full/disconnect handling separate from fallback helpers. Reusing shared send-attempt handling reduces duplicate branch logic and keeps liveness state transitions consistent.

## Result

- Centralized primary relay send full/disconnect handling.
- Preserved fallback routing, disconnect-driven hint invalidation, and next-live lookup refresh behavior.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
