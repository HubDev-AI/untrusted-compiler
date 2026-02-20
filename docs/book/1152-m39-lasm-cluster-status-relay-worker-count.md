# 1152 M39 Slice: LASM Cluster Status Relay Worker Count

This slice extends LASM cluster status telemetry with relay worker pool visibility.

## What changed

1. Added `relayWorkerCount` field to cluster status JSON payload.
2. Wired resolved relay worker count from cluster runtime setup into status writer path.
3. Kept existing status fields and write cadence unchanged.

## Why

Cluster status telemetry already reported backend worker and accept-worker counts, but not relay queue worker count.

Adding `relayWorkerCount` makes relay sizing/debug decisions observable in one place without inspecting flags/env or inferring from runtime behavior.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
