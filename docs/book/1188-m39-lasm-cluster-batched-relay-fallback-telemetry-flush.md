# 1188 M39 Slice: LASM Cluster Batched Relay Fallback Telemetry Flush

This slice reduces telemetry overhead on fallback-heavy dispatch paths.

## What changed

1. Added local batching helper for relay fallback telemetry flush:
   - `flush_lasm_cluster_dispatch_fallback_total(...)`
2. Accept loops now accumulate fallback misses in a local counter.
3. Local fallback totals are flushed periodically (and on error/exit) into shared `relayDispatchFallbackTotal`.

## Why

Fallback telemetry remained useful, but per-fallback atomic increments add overhead under heavy fallback pressure. Local batching preserves monotonic telemetry semantics while reducing atomic update frequency.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
