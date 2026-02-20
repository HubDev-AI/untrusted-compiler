# 1177 M39 Slice: LASM Cluster Single-Relay Dispatch-Counter Bypass

This slice removes unnecessary accept-loop dispatch-counter atomics in single-relay configurations.

## What changed

1. `run_lasm_cluster_accept_loop` now computes `relay_dispatch_uses_counter = relay_sender_count > 1`.
2. Dispatch-counter reservation (`fetch_add`) now runs only when multiple relay senders exist.
3. Single-relay (`1`) and no-relay (`0`) sender modes keep dispatch start index fixed at `0`.

## Why

Single-relay mode does not need round-robin sender distribution; dispatch always targets index `0`. Reserving dispatch-counter ranges in that mode adds avoidable shared atomic churn on the accept hot path.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
