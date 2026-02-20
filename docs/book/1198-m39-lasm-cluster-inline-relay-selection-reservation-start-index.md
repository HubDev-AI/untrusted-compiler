# 1198 M39 Slice: LASM Cluster Inline Relay Selection Reservation Start Index

This slice removes closure indirection from the relay backend-selection hot path.

## What changed

1. In relay worker backend selection, replaced per-connection closure-based start-index computation with inline logic.
2. Reservation refill, offset progression, and wrap behavior remain unchanged.
3. Identity vs lookup-table backend selection behavior remains unchanged.

## Why

The closure was created and invoked on every connection selection step. Inlining this logic reduces hot-path indirection and keeps the selection flow simpler and more explicit.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
