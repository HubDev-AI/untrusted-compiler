# M39: LASM Cluster Relay Pump Budget Local Count

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `/Users/vladimirtrifonov/src/ai/AILang/compiler/sec4-cli/src/lasm_cluster_relay_worker_loop.rs`:
  - budgeted relay pump scheduler now tracks mutable local `relay_count`,
  - pre-step cursor progression now uses local `relay_len_before_step = relay_count`,
  - removed-path cursor normalization now reuses local `relay_count` after decrement instead of re-reading vector length.

## Why

Budgeted relay pump path still performed repeated vector length reads after each pump step, especially on removed-path normalization. Local relay-count tracking keeps scheduler semantics intact while reducing repeated length reads in hot-path iterations.

## Result

- Budgeted relay pump loop uses local relay-count state for step progression and removed-path normalization.
- Deterministic cursor wrap/remove semantics remain unchanged.
- No change to warning, release, or counter-flush behavior.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
