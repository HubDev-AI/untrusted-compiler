# 1263 M39 Slice: LASM Degraded Live-Cursor Realignment

This slice optimizes degraded relay-shard dispatch behavior in LASM cluster accept loop.

## What changed

1. Added `realign_lasm_cluster_dispatch_cursor_to_live(...)` helper.
2. Multi-sender accept dispatch now short-circuits immediately to unavailable handling when no live relay shards remain (`relay_live_sender_count == 0`).
3. After disconnect handling in primary/fallback dispatch paths, when degraded mode is active, the accept loop now realigns `relay_dispatch_cursor` to a live shard once (if available).

## Why

In degraded states, repeated request-by-request live-shard lookup scans add avoidable overhead. Early no-live short-circuit and one-time cursor realignment reduce repeated scan cost while keeping deterministic unavailable/saturation behavior unchanged.

## Validation

1. `rustfmt compiler/sec4-cli/src/main.rs`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
