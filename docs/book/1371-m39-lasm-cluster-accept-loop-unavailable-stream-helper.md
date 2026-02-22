# M39: LASM Cluster Accept-Loop Unavailable-Stream Helper

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `/Users/vladimirtrifonov/src/ai/AILang/compiler/sec4-cli/src/lasm_cluster_accept_loop.rs`:
  - added helper `handle_lasm_cluster_accept_unavailable_stream(...)`,
  - accept-loop degraded branches now reuse this helper for unavailable-stream dispatch handling.

## Why

Unavailable-stream handling in degraded paths was repeated in several branches with identical parameters. One helper keeps this behavior centralized and reduces branch-body duplication in the accept loop.

## Result

- Centralized unavailable-stream dispatch handling in accept loop.
- Reduced repeated inline error-handling blocks in degraded routing branches.
- No change to deterministic saturation/fallback/short-circuit counter behavior.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
