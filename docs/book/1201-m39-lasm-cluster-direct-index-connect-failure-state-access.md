# 1201 M39 Slice: LASM Cluster Direct Index Connect-Failure State Access

This slice tightens relay connect-failure state updates.

## What changed

1. Replaced `get_mut()`-based unhealthy cooldown updates with direct index access on `unhealthy_ports_until_by_index`.
2. Replaced `get()/get_mut()`-based warning-throttle checks/updates with direct index access on `connect_warning_next_allowed_by_index`.

## Why

The selected backend index is already validated by selection logic and vector sizing. Direct index access preserves behavior while removing repeated option chaining in the relay connect-failure path.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
