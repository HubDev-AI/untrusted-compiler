# 1206 M39 Slice: LASM Cluster Relay/Autoscale Direct Match Checks

This slice removes remaining map/unwrap option-chain checks from cluster relay and autoscale paths.

## What changed

1. Relay unhealthy-prune gate now checks `unhealthy_prune_next_at` via direct `match`.
2. Relay connect-failure unhealthy-slot marking now checks the slot with direct `match`.
3. Autoscale up/down cooldown checks now use direct `match`-based elapsed guards instead of map/unwrap chains.

## Why

These checks execute frequently in relay and autoscale loops. Direct `match` checks keep logic explicit and remove option-chain overhead.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
