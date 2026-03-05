# 1238 M39 Slice: LASM Cluster Status Snapshot Pointer-Aware Equality

This slice optimizes status snapshot unchanged detection for worker-port snapshots.

## What changed

1. Replaced derived `PartialEq` on `LasmClusterStatusSnapshot` with manual implementation.
2. Worker-port equality now short-circuits on pointer identity:
   - `Arc::ptr_eq(&self.worker_ports, &other.worker_ports)`
3. If pointer identity differs, equality falls back to slice-value comparison for correctness.

## Why

Status writer compares snapshots on every status interval. Worker-port snapshots are often pointer-stable (`ArcSwap` unchanged), so pointer-aware equality avoids repeated deep vector comparisons while preserving deterministic unchanged-snapshot behavior.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
3. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
