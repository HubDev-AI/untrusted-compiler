# 1262 M39 Slice: LASM Two-Sender Fallback Fast Path

This slice optimizes fallback dispatch in the common two-relay-sender LASM cluster configuration.

## What changed

1. Added a specialized `sender_count == 2` branch in `dispatch_lasm_cluster_relay_stream_fallback_multi`.
2. In two-sender mode, fallback now does one direct alternate-shard `try_send` attempt instead of entering the generic scan loop.
3. Preserved existing behavior:
   - disconnected alternate shard still updates live-shard tracking and `relay_all_senders_live`,
   - fallback still returns deterministic `Saturated` vs `Unavailable` outcomes based on observed live-sender state.

## Why

The generic fallback loop is designed for multi-shard pools, but two-sender clusters are common and only need one alternate send attempt. A direct path removes unnecessary loop bookkeeping in that case while preserving the same runtime semantics.

## Validation

1. `rustfmt compiler/sec4-cli/src/main.rs`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
