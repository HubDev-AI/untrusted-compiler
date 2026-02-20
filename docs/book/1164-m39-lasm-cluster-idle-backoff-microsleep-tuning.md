# 1164 M39 Slice: LASM Cluster Idle Backoff Micro-Sleep Tuning

This slice tunes idle-loop backoff cadence in LASM cluster accept/relay loops.

## What changed

1. Added shared idle-loop constants:
   - `LASM_CLUSTER_IDLE_SPIN_THRESHOLD` (`32`),
   - `LASM_CLUSTER_IDLE_SLEEP_MICROS` (`250`).
2. Updated accept-loop idle behavior:
   - after threshold spins, sleep now uses `Duration::from_micros(250)` instead of `1ms`.
3. Updated relay-worker idle behavior the same way:
   - shared threshold/sleep constants applied in relay idle path.

## Why

A 1ms idle sleep can add avoidable wake-up latency under bursty traffic. Using a bounded micro-sleep keeps busy-loop protection while improving responsiveness when new connections arrive shortly after idle periods.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
