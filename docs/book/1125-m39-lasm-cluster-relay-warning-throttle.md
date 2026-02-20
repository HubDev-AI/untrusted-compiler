# 1125 M39 Slice: LASM Cluster Relay Warning Throttle

This slice reduces relay failure-path logging overhead under cluster churn/saturation.

## What changed

1. Added relay warning throttle constant:
   - `LASM_CLUSTER_RELAY_WARNING_THROTTLE_MS = 1000`
2. Added per-worker warning throttling for backend connect failures:
   - warnings for `LASM cluster worker <port> connect failed` are now rate-limited per backend port.
3. Added shared warning throttling for relay init/pump failures:
   - `LASM cluster relay init failed`
   - `LASM cluster relay pump failed`
   - these are now rate-limited per relay worker loop.

## Why

Under heavy worker churn or repeated backend connect failures, unbounded warning logs can dominate runtime cost and pollute operator output.

Rate-limiting warning emission preserves diagnostics while preventing stderr log spam from becoming a throughput bottleneck in cluster failure paths.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
3. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_sqlite_records_and_query_one_when_db_intrinsics_are_used`
