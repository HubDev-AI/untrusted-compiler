# 1261 M39 Slice: LASM All-Live Relay Fast Path

This slice adds a healthy-pool fast path to LASM cluster accept dispatch.

## What changed

1. Added `relay_all_senders_live` state to multi-sender accept loop dispatch.
2. Primary dispatch selection behavior now splits:
   - when all relay shards are known live: dispatch uses direct `relay_dispatch_cursor` (no live-scan lookup),
   - after first observed sender disconnect: loop switches to live-shard lookup mode.
3. Disconnect handling (`TrySendError::Disconnected`) now also flips `relay_all_senders_live = false` in both primary and fallback dispatch paths.

## Why

After relay shard liveness tracking was introduced, each request still paid live-shard lookup overhead even in the common healthy case. This fast path keeps the healthy case on a direct cursor path and only enables scan-based selection after real shard degradation is detected.

## Validation

1. `rustfmt compiler/sec4-cli/src/main.rs`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
