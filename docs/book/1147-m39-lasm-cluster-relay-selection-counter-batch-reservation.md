# 1147 M39 Slice: LASM Cluster Relay Selection Counter Batch Reservation

This slice reduces backend-selection atomic churn in LASM cluster relay workers.

## What changed

1. Added relay worker-local reservation state for backend-selection counter slots.
2. In healthy-port fast path, relay workers now reserve `relay_accept_batch_max` slots via one atomic `fetch_add(...)` call.
3. Per-connection backend selection now consumes reserved slots locally instead of issuing atomic `fetch_add(1)` for every accepted socket.
4. Unhealthy-port fallback selection logic remains unchanged.

## Why

Under heavy relay load, backend selection performed one shared atomic increment per accepted socket.

Batch reservation keeps the same round-robin semantics while reducing high-frequency atomic contention across relay worker threads.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
3. `benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh --duration 20s --target-requests 600000 --skip-build --out benchmark-suite/results/m39-cluster-probe-selection-reservation-20260220-185008.json`
