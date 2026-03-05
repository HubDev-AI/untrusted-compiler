# M39: LASM Fallback Error Enum Module Ownership

Date: 2026-02-21  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated:
  - `$REPO_ROOT/compiler/sec4-cli/src/lasm_cluster_fallback_dispatch.rs`
  - `$REPO_ROOT/compiler/sec4-cli/src/lasm_cluster_relay_send.rs`
  - `$REPO_ROOT/compiler/sec4-cli/src/main.rs`
- Moved `LasmClusterRelayDispatchError` definition from `main.rs` into `lasm_cluster_fallback_dispatch.rs`.
- Updated main/send modules to import enum ownership from fallback module.

## Why

After fallback dispatch extraction, the error enum still lived in `main.rs`, leaving fallback type ownership split across modules. Moving enum ownership into fallback module aligns type ownership with the fallback implementation boundary.

## Result

- Fallback error type now lives with fallback dispatch implementation.
- Cleaner module boundaries with no behavior change.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
