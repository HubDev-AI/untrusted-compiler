# 1236 M39 Slice: LASM Cluster Selection-Lookup All-Unhealthy Fast Fail

This slice adds an early fast-fail path for the all-unhealthy backend state.

## What changed

1. `rebuild_lasm_cluster_backend_selection_lookup(...)` now returns immediately when `unhealthy_port_count >= worker_port_count`.
2. In that state, lookup rebuild skips full unhealthy-index scanning and returns `(false, false)` directly.

## Why

When all backends are unhealthy, there is no valid routing target. The previous logic still entered lookup-building scan loops before converging on no-healthy result. Fast-failing avoids unnecessary work in overload/failure conditions.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
3. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
