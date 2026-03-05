# 1126 M39 Slice: LASM Cluster Relay Accept-Batch Fairness

This slice improves LASM cluster relay-loop fairness under sustained incoming connection pressure.

## What changed

1. Extended cluster runtime config with relay accept-batch cap:
   - `cluster_relay_accept_batch_max`
2. Added deterministic env resolver:
   - `resolve_lasm_cluster_relay_accept_batch_max()`
   - reads `SEC4_RT_LASM_CLUSTER_RELAY_ACCEPT_BATCH_MAX`
   - default `64`, clamped `1..4096`
3. Wired the resolved value into `LasmClusterConfig` for cluster-mode runs.
4. Updated relay worker loop behavior:
   - each worker now accepts up to the configured batch size per cycle,
   - then returns to pumping active relays before the next accept cycle.
5. Cluster status JSON now includes `relayAcceptBatchMax`:
   - exposes the effective accept-batch cap in runtime telemetry snapshots for operator verification.

## Why

Under continuously full accept queues, an unbounded accept drain can starve pumping of already-active relay pairs.

Bounding accepts per worker cycle forces fair scheduling between intake and in-flight relay pumping, reducing head-of-line stall risk while keeping runtime behavior deterministic and operator-tunable.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
