# 1233 M39 Slice: LASM Cluster Trivial Worker-Count Selection-Lookup Bypass

This slice avoids unnecessary selection-lookup rebuild work for trivial worker counts.

## What changed

1. In relay worker selection-refresh logic, worker-port counts `0` and `1` now bypass `rebuild_lasm_cluster_backend_selection_lookup(...)`.
2. Trivial counts now set deterministic selection flags directly:
   - `worker_port_count == 0` => no healthy backends,
   - `worker_port_count == 1 && unhealthy_port_count == 0` => healthy identity path,
   - `worker_port_count == 1 && unhealthy_port_count > 0` => no healthy backends.
3. Lookup vector storage is cleared for trivial-count cases.

## Why

Lookup rebuild logic is only needed for multi-backend selection mapping. Skipping rebuild for trivial counts removes unnecessary work in a high-frequency loop while preserving backend-selection behavior.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
3. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
