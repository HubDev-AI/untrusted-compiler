# 1184 M39 Slice: Cluster Status Relay Backend Connect Timing Fields

This slice extends LASM cluster status telemetry with relay backend connect timing settings.

## What changed

1. `write_lasm_cluster_status_json` now includes:
   - `relayBackendConnectTimeoutMs`
   - `relayBackendConnectCooldownMs`
2. Status-writer thread now emits those values from active cluster config (`LasmClusterConfig`).

## Why

Cluster tuning runs now have direct visibility into effective relay backend connect timeout/cooldown values through status JSON, which reduces ambiguity when comparing load/perf profiles across runs.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
