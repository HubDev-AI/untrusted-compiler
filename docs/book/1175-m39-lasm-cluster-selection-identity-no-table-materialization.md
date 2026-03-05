# 1175 M39 Slice: LASM Cluster Selection Identity No-Table Materialization

This slice removes unnecessary lookup-table writes in healthy identity-selection mode.

## What changed

1. In `rebuild_lasm_cluster_backend_selection_lookup`, identity mode now returns early without resizing/filling lookup table.
2. Relay lookup rebuild gating now skips lookup-length checks while identity mode is active.
3. Selection behavior remains unchanged:
   - identity mode selects backend index directly from `start_index`,
   - unhealthy mode continues to use explicit lookup table remapping.

## Why

Identity mode is the common healthy steady-state path. Materializing a full lookup table in this mode adds avoidable O(n) writes during rebuild events. Skipping table materialization reduces rebuild overhead while preserving semantics.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
