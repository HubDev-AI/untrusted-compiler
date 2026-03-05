# 1207 M39 Slice: LASM Cluster Duration Reuse in Relay/Autoscale Loops

This slice reduces repeated `Duration::from_millis(...)` construction in frequently executed cluster loops.

## What changed

1. Relay worker loop now precomputes warning-throttle and unhealthy-prune durations once and reuses them.
2. Status writer loop now precomputes status interval duration and reuses it for loop sleep.
3. Autoscale loop now precomputes check + cooldown durations and reuses them for cooldown checks and evaluation cadence.

## Why

These values are loop-invariant for a running process. Reusing precomputed durations removes repeated conversion work in hot runtime paths while preserving existing semantics.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
