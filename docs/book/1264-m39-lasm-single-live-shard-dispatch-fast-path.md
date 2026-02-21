# 1264 M39 Slice: LASM Single-Live-Shard Dispatch Fast Path

This slice optimizes degraded LASM cluster accept dispatch when exactly one relay shard remains live.

## What changed

1. Added `refresh_lasm_cluster_single_live_sender_index(...)` helper.
2. Accept loop now tracks `relay_single_live_sender_index` in multi-sender mode.
3. In degraded mode:
   - if `relay_live_sender_count == 0`, accept dispatch fast-fails to deterministic unavailable handling,
   - if `relay_live_sender_count == 1`, dispatch cursor is pinned directly to the cached single live shard index.
4. Single-live index cache is refreshed on disconnect transitions and after fallback dispatch updates.

## Why

With one surviving relay shard, repeated generic live-shard scans are unnecessary work on every request. Caching and pinning to the single live shard removes that repeated scan overhead while keeping deterministic failure semantics unchanged.

## Validation

1. `rustfmt compiler/sec4-cli/src/main.rs`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
