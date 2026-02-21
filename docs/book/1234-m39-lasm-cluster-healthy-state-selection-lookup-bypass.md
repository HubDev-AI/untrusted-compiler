# 1234 M39 Slice: LASM Cluster Healthy-State Selection-Lookup Bypass

This slice removes selection-lookup rebuild work from the common healthy relay state.

## What changed

1. Relay selection refresh now fast-paths `unhealthy_port_count == 0` for any non-zero worker-port count.
2. In healthy state, loop sets deterministic selection flags directly (`selection_has_healthy_backends = true`, `selection_lookup_is_identity = true`) and clears lookup storage.
3. `rebuild_lasm_cluster_backend_selection_lookup(...)` now runs only when there are unhealthy ports and more than one backend worker.

## Why

Healthy backend state is the dominant runtime path under normal operation. Bypassing lookup rebuild in this case removes unnecessary work from high-frequency relay dispatch checks while preserving selection behavior.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
3. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
