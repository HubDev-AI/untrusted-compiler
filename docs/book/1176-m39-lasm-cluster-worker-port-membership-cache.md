# 1176 M39 Slice: LASM Cluster Worker-Port Membership Cache

This slice optimizes worker-topology membership checks used in prune paths.

## What changed

1. Added helper `rebuild_lasm_cluster_worker_port_membership_set(...)`.
2. Relay workers now maintain a cached worker-port membership set (`HashSet<u16>`) alongside snapshot/addrs state.
3. On snapshot changes, relay workers rebuild:
   - backend address cache,
   - worker-port membership set cache.
4. Unhealthy-port prune and connect-warning prune paths now use membership-set `contains` checks.

## Why

Prune paths may check many map entries against current worker ports. Using a cached membership set avoids repeated per-entry `binary_search` calls over worker snapshots and keeps membership checks fast under topology churn.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
