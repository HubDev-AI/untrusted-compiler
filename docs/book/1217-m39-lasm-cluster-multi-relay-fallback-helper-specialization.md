# 1217 M39 Slice: LASM Cluster Multi-Relay Fallback Helper Specialization

This slice narrows fallback-dispatch helper scope to the actual multi-relay call path.

## What changed

1. Renamed fallback helper to `dispatch_lasm_cluster_relay_stream_fallback_multi`.
2. Removed single-sender guard branch from fallback helper and replaced with `debug_assert!(sender_count > 1)`.
3. Multi-relay accept path now calls the specialized helper directly.

## Why

Fallback scan helper is invoked only from multi-relay accept dispatch. Specializing it for that mode removes an unnecessary runtime guard branch from the fallback hot path while preserving error classification semantics.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
