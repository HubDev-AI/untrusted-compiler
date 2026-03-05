# M39: LASM Cluster Relay Worker Direct Wrap Cursor Progression

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `$REPO_ROOT/compiler/sec4-cli/src/lasm_cluster_relay_worker_loop.rs`:
  - relay selection reservation now advances `relay_selection_reservation_next_index` with direct increment+wrap arithmetic,
  - relay pump loop (`relay_connections.len() > relay_pump_batch_max`) now advances `relay_pump_cursor` with direct increment+wrap on `Progressed` and `Idle` steps,
  - removed wrapped-index helper dependency from this relay-worker hot path.

## Why

Both cursor-progress paths execute repeatedly under relay load. Inlining increment+wrap keeps behavior identical while removing helper-call overhead from frequently executed selection/pump steps.

## Result

- Reduced helper indirection in relay-worker selection and pump cursor progression.
- No change to backend-selection order or relay pump step semantics.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
