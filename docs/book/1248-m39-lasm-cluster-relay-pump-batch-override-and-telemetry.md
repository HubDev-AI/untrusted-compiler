# 1248 M39 Slice: LASM Cluster Relay Pump-Batch Override + Telemetry

This slice makes relay pump-batch sizing explicitly tunable and visible in runtime/probe telemetry so hot-path tuning can be done without code edits.

## What changed

1. Added LASM cluster CLI flag:
   - `--cluster-relay-pump-batch-max <n>`
2. Added deterministic guard behavior for the new flag:
   - rejects zero (`must be >= 1`),
   - LASM-only,
   - cluster-mode-only (`--instances > 1`),
   - rejected in fixed reuse-port mode (proxy relay path not used there).
3. Added runtime resolution support:
   - resolver function `resolve_lasm_cluster_relay_pump_batch_max(...)`,
   - env override support through `SEC4_RT_LASM_CLUSTER_RELAY_PUMP_BATCH_MAX`,
   - default derived from accept-batch baseline (`accept_batch * 4`, bounded).
4. Added cluster status telemetry field:
   - `relayPumpBatchMax`
5. Extended capacity probe script:
   - new flag/env input: `--cluster-relay-pump-batch-max` / `LASM_CAPACITY_CLUSTER_RELAY_PUMP_BATCH_MAX`,
   - dry-run plan output now includes `clusterRelayPumpBatchMax`,
   - summary JSON now includes:
     - `run.clusterRelayPumpBatchMax`,
     - `run.clusterRelayPumpBatchMaxResolved`.

## Why

Relay pump-batch sizing directly impacts proxy relay scan behavior under high active-connection fan-in. Exposing this as a first-class runtime tuning knob avoids repeated code edits and keeps tuning reproducible through benchmark artifacts.

## Validation

1. `cargo test -p sec4 --test commands run_command_rejects_cluster_relay_accept_batch_max_with_c_backend`
2. `cargo test -p sec4 --test commands run_command_rejects_cluster_relay_queue_without_cluster_mode`
3. `bash -n benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh`
4. `bash -n benchmark-suite/scripts/test_run_lasm_cluster_capacity_probe.sh`
5. `benchmark-suite/scripts/test_run_lasm_cluster_capacity_probe.sh`
6. `benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh --skip-build --duration 5s --threads 2 --connections 64 --target-requests 50000 --cluster-relay-pump-batch-max 512 --out results/summaries/sec4-lasm-cluster-capacity-probe-pump-batch-smoke.json`
