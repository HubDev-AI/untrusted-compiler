# 1241 M39 Slice: LASM Cluster Capacity Probe Accept-Worker/Batch Overrides

This slice extends the LASM cluster capacity probe tooling so relay accept-path knobs can be tuned directly from probe CLI arguments.

## What changed

1. Added new optional flags to `benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh`:
   - `--cluster-accept-workers <n>`
   - `--cluster-relay-accept-batch-max <n>`
2. Wired both flags into:
   - numeric argument validation,
   - dry-run plan output,
   - forwarded `sec4 run --backend lasm` arguments,
   - output summary JSON (`run.clusterAcceptWorkers`, `run.clusterRelayAcceptBatchMax`).
3. Extended script contract test coverage (`benchmark-suite/scripts/test_run_lasm_cluster_capacity_probe.sh`) to assert both new dry-run plan fields.

## Why

The scaling lane needs faster, reproducible tuning loops without relying on ad-hoc environment overrides. Exposing accept-worker and accept-batch controls directly in probe CLI keeps performance exploration deterministic and scriptable for CI/operator comparisons.

## Validation

1. `bash -n benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh`
2. `bash -n benchmark-suite/scripts/test_run_lasm_cluster_capacity_probe.sh`
3. `benchmark-suite/scripts/test_run_lasm_cluster_capacity_probe.sh`
