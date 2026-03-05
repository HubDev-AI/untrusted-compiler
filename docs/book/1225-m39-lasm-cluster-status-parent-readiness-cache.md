# 1225 M39 Slice: LASM Cluster Status Parent Readiness Cache

This slice reduces repeated parent-directory syscalls in cluster status writes.

## What changed

1. Added cached parent-readiness state for status writer (`status_parent_ready`).
2. Status writer now initializes parent directory only when readiness is false.
3. Temp-file create path now handles `NotFound` by clearing readiness, re-creating parent directory, and retrying file creation once.

## Why

Changed-snapshot status writes previously called `create_dir_all` every time even after the parent directory was known to exist. Caching readiness removes repeated directory checks while preserving recovery if the directory is removed during runtime.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
3. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
