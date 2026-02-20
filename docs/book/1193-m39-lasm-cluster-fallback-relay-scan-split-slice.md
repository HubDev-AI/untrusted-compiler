# 1193 M39 Slice: LASM Cluster Fallback Relay Scan Split Slice

This slice reduces index arithmetic in fallback relay dispatch scanning.

## What changed

1. Updated `dispatch_lasm_cluster_relay_stream_fallback` to iterate fallback senders using split-slice traversal:
   - `relay_senders[start_index..]`
   - `relay_senders[..start_index]`
2. Kept scan bounded by `sender_count - 1` attempts so the already-attempted preferred shard is still skipped.
3. Preserved existing saturated vs unavailable fallback error semantics.

## Why

The previous fallback scan used per-iteration modulo/wrap index updates. Split-slice traversal keeps the same ordering while reducing inner-loop index arithmetic overhead.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
