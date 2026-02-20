# 1209 M39 Slice: LASM Cluster Relay Vector Preallocation

This slice reduces allocation churn in relay worker startup and steady-state growth.

## What changed

1. Relay worker now initializes `relay_connections` with capacity derived from `relay_accept_batch_max`.
2. Relay worker now initializes `relay_buffer_pool` with `relay_buffer_pool_max` capacity.
3. Initial backend address vector now preallocates to current worker-port snapshot length before the first rebuild.

## Why

These vectors are long-lived and grow toward predictable bounds during cluster operation. Preallocating with known capacity reduces early runtime growth reallocations on hot relay paths.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
