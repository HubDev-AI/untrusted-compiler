# 1132 M39 Slice: LASM Cluster Unhealthy Membership Set Prune

This slice improves relay unhealthy-map pruning efficiency in LASM cluster mode.

## What changed

1. Updated relay unhealthy-port pruning logic to use a per-cycle active worker-port set.
2. In cycles with non-empty worker snapshot:
   - active worker ports are collected once into `HashSet<u16>`,
   - unhealthy map retain checks use set membership instead of repeated linear `Vec::contains` scans.
3. In cycles with empty worker snapshot:
   - unhealthy cooldown map is cleared immediately.

## Why

The prior membership-aware pruning used `worker_ports.contains(...)` per unhealthy-map entry, which scales linearly with worker count for each retain check.

Using a per-cycle set keeps pruning deterministic while reducing repeated membership-scan overhead in larger cluster worker sets.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
