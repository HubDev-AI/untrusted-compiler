# 1220 M39 Slice: LASM Cluster Status Writer Typed Snapshot Compare

This slice optimizes unchanged-snapshot detection in cluster status writing.

## What changed

1. Added typed in-memory snapshot model: `LasmClusterStatusSnapshot`.
2. Status writer now compares `Option<LasmClusterStatusSnapshot>` values to detect unchanged state.
3. Removed per-interval baseline JSON payload encoding used previously for unchanged-state comparison.
4. Kept existing behavior:
   - unchanged snapshots skip status file rewrite,
   - changed snapshots write atomically with fresh `updatedAtMs`.

## Why

Baseline-byte comparison required JSON encoding every interval even when no status fields changed. Typed snapshot comparison moves unchanged detection to lightweight in-memory equality and keeps encoding only for actual file writes.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
3. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
