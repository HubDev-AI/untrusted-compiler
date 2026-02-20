# 1208 M39 Slice: LASM Cluster Accept Fast Path Direct Enqueue

This slice tightens accept-loop fast-path behavior in LASM cluster proxy mode.

## What changed

1. Single-relay accept path now increments `listener_enqueued_local` directly when `try_send` succeeds.
2. Multi-relay accept path now increments `listener_enqueued_local` directly when the primary `try_send` succeeds.
3. `handle_lasm_cluster_accept_dispatch_result(...)` is now used only for saturation/unavailable fallback paths.

## Why

Immediate `try_send` success is the common case in steady-state traffic. Bypassing helper dispatch handling on that path removes one helper call and one result wrapper from each accepted fast-path request while preserving fallback/error semantics.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
