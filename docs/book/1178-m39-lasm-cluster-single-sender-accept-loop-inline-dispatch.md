# 1178 M39 Slice: LASM Cluster Single-Sender Accept-Loop Inline Dispatch

This slice removes generic-dispatch call overhead from single-relay accept-loop paths.

## What changed

1. `run_lasm_cluster_accept_loop` now detects `relay_sender_count == 1` and dispatches directly through that sender.
2. Single-sender path performs direct `try_send` mapping:
   - `Ok(())` -> dispatched,
   - `TrySendError::Full` -> `RelaySaturated`,
   - `TrySendError::Disconnected` -> `RelayUnavailable`.
3. Multi-sender paths continue to use `dispatch_lasm_cluster_relay_stream` unchanged.

## Why

In single-relay deployments, generic sender-rotation logic is unnecessary. Inlining dispatch at the accept loop removes one hot-path call/branch layer while preserving overload/error semantics.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
