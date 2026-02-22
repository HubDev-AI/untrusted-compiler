# M39: LASM Cluster Accept-Loop Post-Batch Flush Helper

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `/Users/vladimirtrifonov/src/ai/AILang/compiler/sec4-cli/src/lasm_cluster_accept_loop.rs`:
  - added `flush_lasm_cluster_accept_post_batch_counters(...)`,
  - moved end-of-batch active/fallback/short-circuit counter flush logic into this helper,
  - accept-loop main body now calls the helper once after accepted-batch handling.

## Why

Post-batch counter flush logic remained inline in the accept-loop body and repeated branch checks for each counter group. A shared helper keeps this flush behavior centralized and simplifies the outer loop.

## Result

- One post-batch flush path for active increments and dispatch totals.
- Accept-loop main body is shorter and easier to maintain.
- No change to flush conditions or counter semantics.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
