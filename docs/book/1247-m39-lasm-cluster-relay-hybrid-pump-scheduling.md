# 1247 M39 Slice: LASM Cluster Relay Hybrid Pump Scheduling

This slice adds a hybrid relay-pump strategy in LASM cluster proxy workers so large active connection sets do not force full-vector scans on every relay loop turn.

## What changed

1. Added relay pump scheduling bounds:
   - `LASM_CLUSTER_RELAY_PUMP_BATCH_MULTIPLIER`
   - `LASM_CLUSTER_RELAY_PUMP_BATCH_MIN`
   - `LASM_CLUSTER_RELAY_PUMP_BATCH_MAX`
2. Relay worker loop now has two modes:
   - full-scan mode for normal active set sizes (`relay_connections.len() <= relay_pump_batch_max`),
   - cursor-batched mode for larger active sets (`relay_connections.len() > relay_pump_batch_max`).
3. Cursor-batched mode keeps progress/failure handling and connection teardown semantics unchanged while bounding per-loop pump work.
4. Full-scan mode intentionally preserves existing behavior/perf shape for common steady-state probe sizes.

## Why

Relay workers previously scanned all active relay pumps each loop turn. That is simple and fast for moderate connection sets, but it scales linearly under large keep-alive fan-in. The hybrid path keeps current behavior where it is already efficient and only applies capped cursor pumping in large-set scenarios.

## Validation

1. `cargo test -p sec4 --test commands run_command_rejects_cluster_relay_queue_without_cluster_mode`
2. `benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh --skip-build --duration 20s --threads 8 --connections 256 --target-requests 1000000`
3. Repeated probe run with identical settings to confirm stable pass behavior.

Observed probe snapshots (same settings):

1. `~75.8k req/s`, `p99 ~7.35ms`, pass.
2. `~75.4k req/s`, `p99 ~6.46ms`, pass.
