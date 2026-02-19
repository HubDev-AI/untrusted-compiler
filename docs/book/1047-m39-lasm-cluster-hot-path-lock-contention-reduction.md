# 1047 M39 Slice: LASM Cluster Hot-Path Lock Contention Reduction

This slice reduces proxy hot-path work in LASM cluster mode so per-connection dispatch spends less time under shared state locks.

## What changed

1. In `compiler/sec4-cli/src/main.rs`, relay worker dispatch no longer:
   - prunes dead workers,
   - spawns recovery workers,
   - mutates round-robin cursor state
   on each incoming connection.
2. Worker pruning + min-instance recovery moved into the background scaler/maintenance loop.
3. Relay backend-port selection now uses an atomic selection counter modulo current worker count, so lock scope is limited to reading active worker ports.
4. `LasmClusterState` no longer stores mutable `round_robin_index`.

## Why

The previous relay path mixed maintenance and dispatch. Under load that increases mutex hold time and creates avoidable contention in the exact path handling incoming sockets. Moving recovery/pruning out of that path keeps request dispatch simpler and cheaper.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`

## Notes

- This slice optimizes lock behavior and worker-selection overhead; it does not claim the 1M req/s target is closed.
- Further gains are expected from deeper proxy/runtime IO-path optimization work.
