# 1129 M39 Slice: LASM Cluster Unhealthy Membership Prune

This slice optimizes unhealthy-worker tracking behavior in the LASM relay loop.

## What changed

1. Updated unhealthy-port pruning to be worker-membership aware.
2. Relay worker cycle now prunes unhealthy-port map entries with two rules:
   - drop expired cooldown entries,
   - drop entries for ports that are not in the current worker snapshot.
3. Added fast reject path when all current workers are unhealthy:
   - backend selection now short-circuits to unavailable handling without full candidate scan when `unhealthy_count >= worker_count`.

## Why

When autoscale changes worker sets, unhealthy cooldown maps can retain stale ports that are no longer active.

Pruning against the current worker snapshot keeps unhealthy tracking precise and allows a deterministic full-unhealthy short-circuit, reducing unnecessary scan work in outage-heavy cycles.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
