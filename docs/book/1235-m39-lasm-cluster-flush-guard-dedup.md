# 1235 M39 Slice: LASM Cluster Flush-Guard Dedup

This slice removes duplicate zero-guard branches around relay/accept flush calls.

## What changed

1. Accept loop now always calls:
   - `flush_lasm_cluster_active_connection_increments(...)`
   - `flush_lasm_cluster_dispatch_fallback_total(...)`
2. Relay worker loop now always calls:
   - `flush_lasm_cluster_saturation_counters(...)`
   - `flush_lasm_cluster_active_connection_decrements(...)`
3. Removed outer `if local_counter > 0` guards where flush helpers already perform the same zero checks internally.

## Why

Hot-loop tails had duplicate branch checks before calling helpers that already check for zero deltas. Deduplicating these guards keeps behavior identical while reducing repeated branch work and keeping tail paths simpler.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
3. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
