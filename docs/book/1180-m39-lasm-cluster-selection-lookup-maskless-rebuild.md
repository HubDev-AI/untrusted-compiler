# 1180 M39 Slice: LASM Cluster Selection Lookup Maskless Rebuild

This slice removes one scratch structure from relay backend-selection lookup rebuild.

## What changed

1. `rebuild_lasm_cluster_backend_selection_lookup` no longer accepts/builds `healthy_mask: Vec<bool>`.
2. Lookup rebuild now finds `first_healthy_index` directly from `worker_ports` and `unhealthy_ports_until`.
3. Sentinel lookup entries are written in reverse index order without a parallel healthy-mask vector.

## Why

Removing the extra healthy-mask vector avoids per-refresh clear/resize/fill work in the relay-selection refresh path while keeping healthy/unhealthy remap behavior unchanged.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
