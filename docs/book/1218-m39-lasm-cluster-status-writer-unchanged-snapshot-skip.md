# 1218 M39 Slice: LASM Cluster Status Writer Unchanged-Snapshot Skip

This slice reduces unnecessary cluster-status file writes.

## What changed

1. `write_lasm_cluster_status_json(...)` now computes a baseline payload with `updatedAtMs=0`.
2. Writer compares the baseline payload bytes with the previous baseline snapshot.
3. If unchanged, writer returns early and skips temporary-file write + rename.
4. On changed status fields, writer sets real `updatedAtMs`, writes atomically, and stores new baseline snapshot.

## Why

Cluster status writer previously rewrote JSON every interval even when no operational fields changed, causing avoidable disk churn. Skipping unchanged snapshots reduces filesystem activity while preserving status updates when real cluster state changes.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
