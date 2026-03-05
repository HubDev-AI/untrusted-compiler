# 1197 M39 Slice: LASM Cluster Relay Selection Reservation Direct Increments

This slice tightens per-connection relay backend-selection reservation arithmetic.

## What changed

1. Replaced saturating arithmetic with direct bounded increments in relay selection reservation tracking:
   - `relay_selection_reservation_offset += 1`
   - `relay_selection_reservation_next_index += 1` (with existing wrap-to-zero behavior)

## Why

These reservation counters are bounded by reservation length and worker-port count, and both are guarded by explicit wrap/reset logic. Direct increments preserve behavior while removing saturating arithmetic from the relay backend-selection hot path.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
