# 1192 M39 Slice: LASM Cluster Accept Loop Counter and Conditional Flush Hot Path

This slice tightens small but frequent operations in the LASM cluster accept loop.

## What changed

1. Switched bounded per-batch hot-path counter increments to direct `+= 1` updates:
   - `listener_accepted_in_batch`
   - `listener_enqueued_local`
   - `listener_dispatch_fallback_total_local`
2. End-of-iteration flush helpers are now called conditionally:
   - flush active-connection increments only when `listener_enqueued_local > 0`
   - flush fallback totals only when `listener_dispatch_fallback_total_local > 0`

## Why

These counters are bounded by accept-batch loop behavior and do not require saturating arithmetic in the hot path. Conditional flush calls avoid unnecessary helper invocation overhead on steady-state iterations where no local values are pending.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
