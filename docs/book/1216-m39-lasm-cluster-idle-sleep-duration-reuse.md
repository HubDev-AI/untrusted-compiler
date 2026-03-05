# 1216 M39 Slice: LASM Cluster Idle Sleep Duration Reuse

This slice removes repeated idle-sleep duration construction from tight cluster loops.

## What changed

1. `run_lasm_cluster_accept_loop` now precomputes `listener_idle_sleep_duration`.
2. Relay worker loop in `cmd_run_lasm_cluster` now precomputes `relay_idle_sleep_duration`.
3. Both loops reuse precomputed values for idle backoff sleeps.

## Why

Idle backoff paths are still exercised frequently under low/variable traffic. Reusing precomputed microsecond sleep durations keeps behavior identical while trimming repeated `Duration::from_micros(...)` construction.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
