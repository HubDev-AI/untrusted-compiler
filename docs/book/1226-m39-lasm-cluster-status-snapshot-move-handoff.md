# 1226 M39 Slice: LASM Cluster Status Snapshot Move Handoff

This slice removes one remaining snapshot clone from changed status-file writes.

## What changed

1. `write_lasm_cluster_status_json` now accepts `LasmClusterStatusSnapshot` by value.
2. The unchanged-snapshot check now compares `last_snapshot` against the by-value snapshot reference.
3. On successful write/rename, writer now moves the snapshot into `last_snapshot` instead of cloning.

## Why

After typed-snapshot comparison, worker-port arc reuse, and streamed writes, changed-snapshot status writes still cloned the full snapshot before storing it. Moving the snapshot into `last_snapshot` keeps the same behavior while removing per-write clone overhead.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
3. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
