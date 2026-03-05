# 1185 M39 Slice: LASM Cluster Preferred-Shard Direct Dispatch Fast Path

This slice reduces multi-relay dispatch overhead on the common accept path.

## What changed

1. In multi-relay mode, accept loop now attempts `try_send` on the preferred relay shard first.
2. On first-attempt failure, fallback dispatch scans only the remaining relay shards.
3. Fallback dispatch preserves existing `Saturated` vs `Unavailable` mapping semantics.

## Why

Most healthy traffic paths enqueue on the preferred shard immediately. Direct-first dispatch removes full multi-sender scan overhead from that common case while keeping deterministic failure routing behavior unchanged.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
