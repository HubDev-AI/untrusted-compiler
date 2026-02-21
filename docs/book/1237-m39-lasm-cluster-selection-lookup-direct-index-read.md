# 1237 M39 Slice: LASM Cluster Selection-Lookup Direct Index Read

This slice simplifies unhealthy-state scanning inside backend selection lookup rebuild.

## What changed

1. Added debug precondition: `unhealthy_ports_until_by_index.len() >= worker_port_count`.
2. Replaced per-iteration optional-index reads (`get(...).and_then(...)`) with direct indexed reads:
   - `unhealthy_ports_until_by_index[index].is_none()`

## Why

Selection lookup rebuild is part of relay dispatch hot path in unhealthy states. Direct indexing with a length precondition removes repeated optional/bounds combinator overhead while preserving semantics.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
3. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
