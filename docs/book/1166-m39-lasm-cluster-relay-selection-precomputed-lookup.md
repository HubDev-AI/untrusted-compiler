# 1166 M39 Slice: LASM Cluster Relay Selection Precomputed Lookup

This slice reduces per-connection backend-selection work in relay workers.

## What changed

1. Added helper `rebuild_lasm_cluster_backend_selection_lookup(...)`:
   - builds a table mapping each selection start index to a selected backend index (or `None`),
   - preserves existing unhealthy-worker skip semantics.
2. Relay worker loop now maintains and reuses lookup state:
   - lookup table is rebuilt only when worker-port snapshot pointer changes or unhealthy-port membership changes,
   - per-connection selection now resolves by O(1) lookup from computed start index.
3. Connect-failure path now marks selection lookup dirty and initializes prune cadence when unhealthy map becomes active.

## Why

Before this slice, unhealthy fallback selection performed per-connection candidate scanning. Under saturation/failure conditions this adds hot-path work on every accepted connection. Rebuilding selection tables only on topology/health changes keeps per-connection selection cheap while retaining deterministic behavior.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
