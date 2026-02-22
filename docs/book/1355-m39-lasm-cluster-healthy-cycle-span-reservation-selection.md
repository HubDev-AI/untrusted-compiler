# M39: LASM Cluster Healthy Cycle Span Reservation Selection

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `/Users/vladimirtrifonov/src/ai/AILang/compiler/sec4-cli/src/lasm_cluster_backend_selection.rs`:
  - lookup rebuild now returns degraded healthy cycle span with `(has_healthy, is_identity, cycle_span)` metadata.
- Updated `/Users/vladimirtrifonov/src/ai/AILang/compiler/sec4-cli/src/lasm_cluster_relay_worker_loop.rs`:
  - relay selection reservation now tracks reservation span independently from total worker count,
  - degraded selection path now advances reservation cursor modulo healthy cycle span, while identity mode continues to use total worker count span.

## Why

Even with cyclic healthy lookup entries, reserving by total worker count can still leak dead-slot cadence into degraded routing. Switching reservation cadence to healthy cycle span keeps degraded scheduling strictly coupled to available healthy backends.

## Result

- Degraded reservation cadence is now independent of dead-worker index layout.
- More stable, deterministic healthy-backend selection under partial failure.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
