# 1162 M39 Slice: LASM Cluster Relay Selection Wrap + Split Scan

This slice tightens relay worker backend-selection math in the hottest per-connection selection path.

## What changed

1. Relay selection reservation now tracks wrapped selection index state:
   - keeps `relay_selection_reservation_next_index` and advances it with wrap increment,
   - recomputes modulo only when reservation refreshes or worker-port count changes.
2. Unhealthy fallback scan now iterates with split slices:
   - scans `worker_ports[start_index..]` then `worker_ports[..start_index]`,
   - removes modulo/index arithmetic from per-candidate fallback scanning.

## Why

Relay backend selection runs for every accepted proxied connection. Reducing per-connection modulo/index math lowers CPU overhead on this hot path while preserving existing selection semantics and unhealthy-port skip behavior.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
