# 1219 M39 Slice: LASM Cluster Status Writer Unchanged-Snapshot Test Lock

This slice adds integration coverage for the status-writer unchanged-snapshot optimization.

## What changed

1. Added command integration test:
   - `run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
2. Test runs LASM cluster mode with:
   - `--cluster-status-json`
   - short status interval (`--autoscale-check-ms 100`)
3. Test asserts `updatedAtMs` stays stable across separated reads when status fields are unchanged.

## Why

The previous slice introduced unchanged-snapshot write skipping in the status writer. This test locks that contract so future refactors do not silently revert to periodic unchanged rewrites.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
3. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
