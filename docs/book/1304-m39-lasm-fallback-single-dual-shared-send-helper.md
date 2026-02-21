# M39: LASM Fallback Single/Dual Shared Send Helper

Date: 2026-02-21  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `/Users/vladimirtrifonov/src/ai/AILang/compiler/sec4-cli/src/main.rs`.
- Refactored:
  - `dispatch_lasm_cluster_relay_stream_fallback_single_live(...)`
  - `dispatch_lasm_cluster_relay_stream_fallback_dual_live(...)`
- Both now delegate send/full/disconnect handling to `attempt_lasm_cluster_relay_send(...)` and keep terminal outcome mapping through `lasm_cluster_fallback_terminal_dispatch_error(...)`.

## Why

Single/dual helper paths still carried duplicated send-attempt logic after shared handling was introduced in fallback-multi branches. Reusing the shared helper keeps one deterministic send-attempt implementation for all fallback paths.

## Result

- Centralized fallback send handling across single/dual/multi dispatch helpers.
- Preserved saturated/unavailable terminal semantics via the same terminal helper.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
