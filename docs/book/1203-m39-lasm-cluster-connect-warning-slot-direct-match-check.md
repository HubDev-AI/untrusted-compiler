# 1203 M39 Slice: LASM Cluster Connect Warning Slot Direct Match Check

This slice simplifies relay connect-failure warning-throttle handling.

## What changed

1. Connect-failure warning throttle now reads/writes the per-backend warning slot through one mutable entry reference.
2. Warning allowance check uses direct `match` on the slot value:
   - `Some(next_allowed_at) => now >= next_allowed_at`
   - `None => true`

## Why

The previous path chained separate indexed option reads/writes. This change keeps behavior the same while reducing repeated indexing and option chaining on the connect-failure path.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
