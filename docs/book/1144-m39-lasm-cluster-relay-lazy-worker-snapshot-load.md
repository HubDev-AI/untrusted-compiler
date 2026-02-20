# 1144 M39 Slice: LASM Cluster Relay Lazy Worker Snapshot Load

This slice reduces relay worker hot-loop snapshot churn in LASM cluster mode.

## What changed

1. Relay worker loops now keep worker-port snapshot loading lazy.
2. Worker-port snapshots are loaded only when needed:
   - when unhealthy-port pruning is active, or
   - when at least one accepted client socket must be routed to a backend worker.
3. Removed unconditional per-cycle worker-port snapshot load from relay worker loop setup.

## Why

Before this slice, every relay worker cycle loaded the worker-port snapshot even when no new client socket was accepted and no unhealthy-port pruning was required.

Lazy loading keeps behavior identical for routing/health pruning while cutting avoidable snapshot operations in the steady-state relay loop.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
3. `benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh --duration 20s --target-requests 600000 --skip-build --out benchmark-suite/results/summaries/sec4-lasm-cluster-capacity-probe-relay-lazy-snapshot.json`
