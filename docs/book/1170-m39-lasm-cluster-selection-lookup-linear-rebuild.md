# 1170 M39 Slice: LASM Cluster Selection Lookup Linear Rebuild

This slice improves relay backend-selection lookup rebuild complexity.

## What changed

1. Replaced scan-per-start-index rebuild logic in `rebuild_lasm_cluster_backend_selection_lookup`.
2. New rebuild flow:
   - compute per-index healthy mask once from worker ports + unhealthy-port map,
   - compute first healthy index,
   - fill lookup table in reverse using next-healthy propagation.
3. Complexity reduced from `O(n^2)` rebuild scans to `O(n)` rebuild work.

## Why

Lookup rebuilds run whenever topology/health changes. At higher worker counts, quadratic rebuild scans add avoidable overhead. Linear rebuild keeps selection semantics intact while improving rebuild efficiency.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
