# 1124 M39 Slice: LASM Cluster Backend Connect Timeout Tuning

This slice tunes LASM cluster relay backend connect behavior for faster failover under worker churn and saturation.

## What changed

1. Extended `LasmClusterConfig` with relay backend connect timing controls:
   - `cluster_backend_connect_timeout_ms`
   - `cluster_backend_connect_cooldown_ms`
2. Added deterministic runtime resolution helpers:
   - `resolve_lasm_cluster_backend_connect_timeout_ms()`
   - `resolve_lasm_cluster_backend_connect_cooldown_ms(timeout_ms)`
3. Added env overrides with bounded parsing:
   - `SEC4_RT_LASM_CLUSTER_BACKEND_CONNECT_TIMEOUT_MS` (default `100`, clamped `25..5000`)
   - `SEC4_RT_LASM_CLUSTER_BACKEND_CONNECT_COOLDOWN_MS` (default `timeout*2` clamped `150..2000`, env clamped `25..10000`)
4. Relay worker threads now use precomputed durations from cluster config:
   - avoids fixed hardcoded timings and keeps timeout/cooldown tuning explicit per run.

## Why

The previous fixed relay backend connect timeout (`250ms`) and cooldown (`500ms`) made failover tuning rigid and increased request stall time when a worker port was unhealthy.

This update keeps behavior deterministic while enabling lower-latency failover defaults and operator-level tuning without changing CLI surface.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
3. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_sqlite_records_and_query_one_when_db_intrinsics_are_used`
