# M39: LASM Cluster Relay Selection and Runtime-Config Direct Arithmetic Batch

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `$REPO_ROOT/compiler/sec4-cli/src/lasm_cluster_relay_worker_loop.rs`:
  - hoisted relay selection reservation chunk (`max(relay_accept_batch_max, relay_selection_reservation_min_chunk)`) out of the per-request selection branch.
- Updated `$REPO_ROOT/compiler/sec4-cli/src/lasm_cluster_backend_selection.rs`:
  - replaced reverse scan + branch loop for post-first-healthy wrap fill with direct tail-slice fill.
- Updated `$REPO_ROOT/compiler/sec4-cli/src/lasm_cluster_runtime_config.rs`:
  - changed desired autoscale instance calculation to direct ceil-division math on active connections with explicit `target_connections_per_instance >= 1` guard.

## Why

These computations execute repeatedly in relay selection and autoscale control paths. Direct arithmetic/slice operations reduce repeated per-iteration helper/math overhead while preserving behavior.

## Result

- Reduced branch/math work in relay selection reservation path.
- Simplified healthy-lookup wrap fill logic.
- Kept desired instance semantics intact with tighter arithmetic.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
