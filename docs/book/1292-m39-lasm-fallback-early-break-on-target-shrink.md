# M39: LASM Fallback Early-Break On Target Shrink

Date: 2026-02-21  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated general degraded scan loop in `dispatch_lasm_cluster_relay_stream_fallback_multi(...)` (`/compiler/sec4-cli/src/main.rs`).
- Added early-break check after dispatch attempt:
  - if dynamic live target shrinks (for example after disconnect) and current `scanned_live` already satisfies the new target, loop exits before post-attempt jump computation.

## Why

After disconnect-driven live-target reductions, the loop could still execute additional jump/index-resolution bookkeeping before exiting on the next while-condition check.

## Result

- Avoided unnecessary post-attempt jump work after target shrink.
- Preserved deterministic bounded scan behavior and fallback result semantics.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
