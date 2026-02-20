# 1222 M39 Slice: LASM Cluster Status Writer Typed Payload Serialization

This slice optimizes status JSON payload construction while preserving output contract.

## What changed

1. Added typed serializable payload model: `LasmClusterStatusPayload`.
2. Replaced dynamic `serde_json::json!` map construction in status writer with typed struct serialization (`serde_json::to_vec`).
3. Kept field naming contract via serde camelCase rename rules and unchanged status keys.

## Why

Dynamic map construction incurs extra allocation and runtime key/value assembly overhead. Typed payload serialization keeps output deterministic while reducing dynamic JSON-building cost in changed-snapshot writes.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
3. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
