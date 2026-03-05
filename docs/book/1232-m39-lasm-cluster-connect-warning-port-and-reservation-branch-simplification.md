# 1232 M39 Slice: LASM Cluster Connect-Warning Port and Reservation Branch Simplification

This slice removes two small redundant operations from relay hot paths.

## What changed

1. Relay worker connect-failure warning now prints `backend_addr.port()` from the already-selected backend address.
2. Removed extra selected-worker-port snapshot indexing for warning port resolution.
3. Reservation refill logic in multi-backend relay selection now assigns `relay_selection_reservation_base % worker_port_count` directly.
4. Removed unreachable `worker_port_count <= 1` branch inside the already multi-backend-only selection path.

## Why

Both removed branches/reads were redundant in the active execution path. Keeping hot-path branches and reads minimal helps maintain predictable latency and avoids unnecessary work in highly repeated request-dispatch loops.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
3. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
