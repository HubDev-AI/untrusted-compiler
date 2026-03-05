# 1173 M39 Slice: LASM Cluster Relay Dispatch Direct Slice Loops

This slice tightens sender iteration in relay dispatch hot paths.

## What changed

1. `dispatch_lasm_cluster_relay_stream` now uses direct loops over sender slices:
   - fast path for `start_index == 0` over full sender slice,
   - otherwise tail then head slice loops.
2. Removed iterator-chain dispatch traversal and disconnected-count arithmetic.
3. Preserved dispatch result semantics:
   - any full sender observed => `Saturated`,
   - all senders disconnected => `Unavailable`.

## Why

Dispatch runs for each accepted connection. Direct slice loops avoid iterator-chain plumbing and counter bookkeeping in the hottest dispatch path while keeping behavior unchanged.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
