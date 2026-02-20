# 1138 M39 Slice: LASM Cluster Status Compact JSON Encoding

This slice reduces periodic cluster status telemetry serialization overhead.

## What changed

1. Updated cluster status writer payload encoding from pretty JSON to compact JSON.
2. `write_lasm_cluster_status_json(...)` now uses `serde_json::to_vec(...)` instead of `serde_json::to_vec_pretty(...)`.
3. Status payload fields and semantics remain unchanged.

## Why

Status telemetry writes run periodically in cluster mode. Pretty encoding adds unnecessary formatting overhead and increases payload size without adding runtime value for machine-consumed status snapshots.

Using compact encoding keeps deterministic JSON content while reducing serialization and write overhead.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
