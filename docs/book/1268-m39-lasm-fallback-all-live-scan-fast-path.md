# 1268 M39 Slice: LASM Fallback All-Live Scan Fast Path

This slice optimizes fallback dispatch scanning for healthy relay pools.

## What changed

1. Updated `dispatch_lasm_cluster_relay_stream_fallback_multi` for `sender_count > 2`:
   - when relay pool is known all-live, fallback now runs a direct scan loop without dead-shard branch checks.
2. Existing degraded behavior is preserved:
   - on observed `TrySendError::Disconnected`, shard is marked dead and fallback transitions to degraded scan behavior on subsequent calls.
   - saturated/unavailable result semantics are unchanged.

## Why

Fallback dispatch is usually less common than primary dispatch, but under pressure it can become hot. When all relay shards are known healthy, dead-shard checks are unnecessary overhead. The all-live fast path removes those checks while preserving degraded correctness.

## Validation

1. `rustfmt compiler/sec4-cli/src/main.rs`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
