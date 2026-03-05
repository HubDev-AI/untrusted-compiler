# 1242 M39 Slice: LASM Saturation Suite Accept-Path Override Forwarding

This slice wires LASM accept-path tuning knobs through the full saturation orchestration chain.

## What changed

1. Extended saturation matrix script (`run_lasm_cluster_saturation_boost_matrix.sh`) to accept and forward:
   - `--cluster-accept-workers`
   - `--cluster-relay-accept-batch-max`
2. Extended saturation bundle wrapper (`run_lasm_cluster_saturation_boost_bundle.sh`) to pass the same flags through to matrix/verify probe runs.
3. Extended full benchmark suite entrypoint (`run_full_benchmark_suite.sh`) with matching saturation flags:
   - `--saturation-cluster-accept-workers`
   - `--saturation-cluster-relay-accept-batch-max`
4. Updated benchmark Makefile saturation targets to expose and forward:
   - `LASM_CAPACITY_CLUSTER_ACCEPT_WORKERS`
   - `LASM_CAPACITY_CLUSTER_RELAY_ACCEPT_BATCH_MAX`
5. Updated script contract tests to lock the new forwarding behavior:
   - matrix/bundle/full-suite dry-run passthrough checks,
   - Makefile target token checks.

## Why

After adding accept-path controls to the base LASM capacity probe (`1241`), saturation and full-suite orchestration still could not drive those knobs end-to-end. This slice closes that gap so scale tuning can be run from top-level benchmark commands without manual script edits.

## Validation

1. `bash -n benchmark-suite/scripts/run_lasm_cluster_saturation_boost_matrix.sh`
2. `bash -n benchmark-suite/scripts/run_lasm_cluster_saturation_boost_bundle.sh`
3. `bash -n benchmark-suite/scripts/run_full_benchmark_suite.sh`
4. `benchmark-suite/scripts/test_run_lasm_cluster_saturation_boost_matrix.sh`
5. `benchmark-suite/scripts/test_run_lasm_cluster_saturation_boost_bundle.sh`
6. `benchmark-suite/scripts/test_run_full_benchmark_suite.sh`
7. `benchmark-suite/scripts/test_makefile_profile_targets.sh`
