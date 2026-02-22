# M39: LASM Cluster Accept-Loop Dispatch Counter Flush Helper

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `/Users/vladimirtrifonov/src/ai/AILang/compiler/sec4-cli/src/lasm_cluster_accept_loop.rs`:
  - added helper `flush_lasm_cluster_accept_dispatch_counters(...)`,
  - both accept-error branches now call this helper instead of repeating inline flush blocks,
  - accept-loop final tail now calls the same helper for dispatch counter flushes.

## Why

Accept-loop had duplicated dispatch counter flush blocks in multiple error paths. One shared helper keeps these flush semantics centralized and reduces repeated branch-body code.

## Result

- One dispatch-counter flush path for accept-loop error/final-tail handling.
- Reduced repeated flush blocks in accept-loop branches.
- No change to deterministic counter flush behavior.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
