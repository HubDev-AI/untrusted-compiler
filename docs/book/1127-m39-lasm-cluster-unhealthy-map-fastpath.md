# 1127 M39 Slice: LASM Cluster Unhealthy-Map Fast Path

This slice reduces relay-loop bookkeeping overhead in LASM cluster mode.

## What changed

1. Updated relay worker loop scheduling around unhealthy-port cooldown tracking.
2. Expired unhealthy-port entries are now pruned once per relay worker cycle:
   - pruning runs before the per-cycle accept batch starts,
   - this replaces per-accepted-connection pruning in the unhealthy-port selection path.
3. Backend selection logic for unhealthy-path workers now consumes the already-pruned cooldown map.
4. Fast path behavior when no unhealthy ports are tracked remains unchanged.

## Why

Under sustained incoming traffic, pruning the unhealthy-port map per accepted connection adds avoidable churn in the relay accept path.

Moving pruning to once per worker cycle keeps cooldown behavior deterministic while reducing repeated map-maintenance work during hot request intake.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
