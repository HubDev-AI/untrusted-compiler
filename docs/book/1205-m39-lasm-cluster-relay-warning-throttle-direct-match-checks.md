# 1205 M39 Slice: LASM Cluster Relay Warning Throttle Direct Match Checks

This slice simplifies relay warning-throttle evaluation paths.

## What changed

1. Relay init-failure warning gate now checks `pump_warning_next_allowed` via direct `match`.
2. Relay pump-failure warning gate now checks `pump_warning_next_allowed` via direct `match`.

## Why

Both paths previously used `map(...).unwrap_or(true)` chains. Direct match checks preserve behavior while reducing option-chain overhead and keeping warning-throttle logic explicit.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
