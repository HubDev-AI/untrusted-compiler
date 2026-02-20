# 1146 M39 Slice: LASM Cluster Shared Accept Dispatch Cursor

This slice improves queue-shard fairness when multiple cluster accept workers run in parallel.

## What changed

1. `run_lasm_cluster_accept_loop` now accepts a shared dispatch counter (`AtomicUsize`).
2. Removed per-accept-worker local dispatch cursor state in accept loops.
3. Queue-shard dispatch start index is now derived from the shared counter once per accepted batch.
4. Main accept loop and spawned accept-worker loops now share the same relay dispatch cursor.

## Why

With parallel accept workers, per-thread local round-robin cursors can drift and bias relay queue-shard selection.

A shared atomic dispatch cursor keeps all accept workers on one deterministic global dispatch sequence while preserving existing backend-selection decoupling.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
3. `benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh --duration 20s --target-requests 600000 --skip-build --out benchmark-suite/results/m39-cluster-probe-dispatch-counter-20260220-184536.json`
