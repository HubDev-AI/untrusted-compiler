# 1053 M39 Slice: LASM Cluster Lock-Free Worker-Port Snapshots

This slice removes relay-path state-lock reads by introducing atomic worker-port snapshots for backend selection.

## What changed

1. Added `arc-swap` dependency to `compiler/sec4-cli` and imported `ArcSwap` in cluster runtime.
2. In `cmd_run_lasm_cluster`, created shared worker-port snapshot:
   - `Arc<ArcSwap<Vec<u16>>>`
3. Relay workers now:
   - read current worker-port snapshot atomically,
   - choose backend port using atomic round-robin counter,
   - skip cluster-state lock access on steady-state dispatch.
4. Autoscale/maintenance loop now refreshes the snapshot after prune/recovery/scale mutations.

## Why

Even with `RwLock`, relay workers still paid lock acquisition cost on every connection. Snapshot-based selection keeps relay dispatch on atomic reads and moves synchronization overhead to lower-frequency maintenance paths.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`

## Notes

- Snapshot staleness is bounded by maintenance loop updates and connect-failure fallback handling.
- This is another incremental hot-path optimization step; overall 1M req/s closure remains open.
