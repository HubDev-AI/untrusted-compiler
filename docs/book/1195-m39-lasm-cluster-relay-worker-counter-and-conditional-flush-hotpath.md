# 1195 M39 Slice: LASM Cluster Relay Worker Counter and Conditional Flush Hot Path

This slice trims relay-worker loop overhead in the LASM cluster proxy hot path.

## What changed

1. Relay-worker loop-local counters now use direct increments:
   - `saturation_events_pending_local`
   - `saturation_events_total_local`
   - `active_connection_decrements_local`
2. Relay-worker end-of-iteration flush calls are now conditional:
   - saturation counters flush only when either local saturation counter is non-zero
   - active decrement flush only when local active-decrement counter is non-zero

## Why

These counters are loop-local and flushed every iteration. Using direct increments plus conditional flush invocation removes extra arithmetic/helper overhead on steady-state iterations where no local updates are pending.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
