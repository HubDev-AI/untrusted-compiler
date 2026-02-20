# 1149 M39 Slice: LASM Cluster Relay Selection Batch Fallback Reuse

This slice extends backend-selection batching to the unhealthy-worker fallback path.

## What changed

1. Introduced a shared `next_selection_index` reservation helper in relay worker backend selection.
2. Healthy-path backend selection continues using batch-reserved selection indices.
3. Unhealthy-worker fallback path now also uses reserved selection indices for `start_index` instead of per-connection `fetch_add(1)`.
4. Unhealthy-port skip loop semantics remain unchanged.

## Why

After batching healthy-path selection, the unhealthy fallback branch still performed one shared atomic increment per accepted connection.

Reusing the same reservation window removes that remaining fallback atomic churn while preserving deterministic backend-selection behavior.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
3. `benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh --duration 20s --target-requests 600000 --skip-build --out benchmark-suite/results/m39-cluster-probe-selection-batch-fallback-20260220-190011.json`
