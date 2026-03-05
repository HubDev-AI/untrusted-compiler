# 1139 M39 Slice: LASM Cluster Status Precompute Tmp Path

This slice removes repeated temp-path construction from periodic status telemetry writes.

## What changed

1. Extended status writer helper signature to accept precomputed temp path.
2. Status thread now computes `<status>.tmp` path once before entering the periodic status loop.
3. Each status write call now reuses the precomputed temp path.

## Why

Status telemetry writes run periodically in cluster mode. Recomputing temp-path extension formatting on each iteration adds avoidable allocation/string work.

Precomputing once keeps behavior deterministic while reducing per-iteration overhead in the status writer loop.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
