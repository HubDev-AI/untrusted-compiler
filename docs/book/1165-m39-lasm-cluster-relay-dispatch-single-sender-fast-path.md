# 1165 M39 Slice: LASM Cluster Relay Dispatch Single-Sender Fast Path

This slice adds a fast path for relay dispatch when only one relay sender is configured.

## What changed

1. `dispatch_lasm_cluster_relay_stream` now checks `sender_count == 1` first.
2. In single-sender mode, dispatch uses a direct `try_send` against that sender and returns the same semantic mapping:
   - `Ok(())` -> dispatched,
   - `TrySendError::Full` -> `Saturated`,
   - `TrySendError::Disconnected` -> `Unavailable`.
3. Existing multi-sender split-slice iteration logic remains unchanged.

## Why

Single-relay-worker deployments are common for smaller setups. This fast path removes unnecessary start-index modulo and iterator-chain overhead while preserving deterministic relay error behavior.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
