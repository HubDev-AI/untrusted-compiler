# 1258 M39 Slice: LASM Relay Shard Liveness Tracking

This slice improves the LASM cluster proxy accept-dispatch hot path by tracking disconnected relay shards and skipping them in primary/fallback dispatch selection.

## What changed

1. Added relay shard liveness indexing for multi-sender accept loops:
   - `run_lasm_cluster_accept_loop` now maintains a per-shard live map and live-shard count.
   - Primary dispatch selection now resolves the next live relay shard index instead of blindly targeting a potentially dead shard.
2. Hardened fallback dispatch with liveness updates:
   - `dispatch_lasm_cluster_relay_stream_fallback_multi` now takes relay shard liveness state.
   - When fallback observes `TrySendError::Disconnected`, it marks that shard as dead and decrements live-shard count.
   - Fallback now returns deterministic `Unavailable` when all shards are known dead.
3. Preserved existing overload semantics:
   - Saturation (`Full`) still maps to the same saturated handling path.
   - Unavailable handling still returns deterministic relay-unavailable behavior.

## Why

Under degraded conditions, repeatedly probing already disconnected relay shards adds avoidable dispatch overhead and contention. Keeping shard-liveness state in the accept loop avoids redundant disconnected send attempts and reduces fallback scan work while preserving deterministic failure semantics.

## Validation

1. `rustfmt compiler/sec4-cli/src/main.rs`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
3. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
