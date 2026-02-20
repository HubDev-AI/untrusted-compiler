# 1150 M39 Slice: LASM Cluster Unhealthy Prune Snapshot Membership

This slice simplifies relay-worker unhealthy-port pruning in the cluster hot loop.

## What changed

1. Removed relay worker-local `active_worker_ports` hash-set cache and snapshot pointer cache.
2. Unhealthy-port prune now checks membership directly against the current worker snapshot (`snapshot.contains(port)`).
3. Existing prune rules are preserved:
   - expired unhealthy windows are removed,
   - ports not present in current worker snapshot are removed.

## Why

The previous path refreshed an extra hash-set cache when worker snapshots changed, then used it only for prune membership checks.

Using the snapshot vector directly removes cache maintenance overhead and keeps prune behavior deterministic.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
3. `benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh --duration 20s --target-requests 600000 --skip-build --out benchmark-suite/results/m39-cluster-probe-unhealthy-prune-snapshot-20260220-190342.json`
