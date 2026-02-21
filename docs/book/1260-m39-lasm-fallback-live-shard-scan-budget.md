# 1260 M39 Slice: LASM Fallback Live-Shard Scan Budget

This slice narrows LASM relay fallback dispatch scanning to known-live shard attempts.

## What changed

1. Updated `dispatch_lasm_cluster_relay_stream_fallback_multi` to track:
   - scanned slot count
   - scanned live shard count
2. Fallback scan loop now stops when either condition is reached:
   - all non-primary slots have been traversed, or
   - all remaining live non-primary shards have been attempted (`relay_live_sender_count - 1`).
3. Existing disconnected-shard marking and saturated/unavailable result semantics are preserved.

## Why

After relay shard liveness tracking was added, fallback still iterated over a fixed slot budget. Under partial shard failure, that keeps spending loop iterations on known-dead slots. Limiting attempts to remaining live shards reduces fallback overhead in degraded conditions without changing deterministic error behavior.

## Validation

1. `rustfmt compiler/sec4-cli/src/main.rs`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
