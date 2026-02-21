# 1224 M39 Slice: LASM Cluster Status Writer Streamed Temp-File Serialization

This slice optimizes changed-snapshot status-file writes.

## What changed

1. Status writer now creates a temporary status file explicitly.
2. Changed payload serialization now streams directly to the temp file with `serde_json::to_writer` via `BufWriter`.
3. Writer flushes temp output before atomic rename to destination path.
4. Existing atomic temp-file replacement flow is preserved.

## Why

Changed-snapshot writes previously encoded into an intermediate `Vec<u8>` and then copied via `fs::write`. Streaming serialization removes that extra allocation/copy path while keeping deterministic payload encoding and atomic file replacement behavior.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
3. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
