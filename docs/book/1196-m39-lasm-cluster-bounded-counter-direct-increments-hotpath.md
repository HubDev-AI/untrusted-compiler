# 1196 M39 Slice: LASM Cluster Bounded Counter Direct Increments Hot Path

This slice removes remaining saturating arithmetic from bounded counters in the LASM cluster ingress/relay hot path.

## What changed

1. Replaced bounded `saturating_add(1)` updates with direct `+= 1` updates for:
   - accept-loop `listener_idle_spins`
   - relay-loop `accepted_in_batch`
   - relay-loop `idle_spins`
   - relay unhealthy/prune local counters bounded by worker snapshot size
   - accept-loop local saturation counters bounded by batched flush behavior

## Why

These counters are bounded by explicit loop limits or snapshot cardinality and reset/flush regularly. Using direct increments removes unnecessary saturating arithmetic overhead in hot loops while preserving behavior.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
