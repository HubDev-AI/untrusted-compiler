# 1249 M39 Slice: LASM Saturation Suite Relay Pump-Batch Forwarding

This slice wires the new LASM relay pump-batch tuning knob through the saturation orchestration stack so operators can tune from one command surface.

## What changed

1. Saturation matrix script forwarding:
   - `run_lasm_cluster_saturation_boost_matrix.sh` now accepts/passes `--cluster-relay-pump-batch-max`.
   - Matrix plan output now includes `clusterRelayPumpBatchMax`.
   - Matrix JSON now includes requested run-level `clusterRelayPumpBatchMax` and per-run resolved `clusterRelayPumpBatchMaxResolved`.
2. Saturation bundle script forwarding:
   - `run_lasm_cluster_saturation_boost_bundle.sh` now accepts/passes `--cluster-relay-pump-batch-max`.
   - Bundle plan output now includes `clusterRelayPumpBatchMax`.
3. Full benchmark suite forwarding:
   - `run_full_benchmark_suite.sh` now accepts `--saturation-cluster-relay-pump-batch-max`.
   - Full-suite saturation lane now passes this value through to bundle/matrix/probe flow.
4. Makefile saturation target plumbing:
   - Added `LASM_CAPACITY_CLUSTER_RELAY_PUMP_BATCH_MAX`.
   - Saturation/full-suite and direct LASM probe/matrix/bundle targets now pass relay pump-batch max.
   - Throughput/latency preset wrappers now forward dedicated preset variables (`LASM_SATURATION_*_RELAY_PUMP_BATCH_MAX`).
5. Script contract coverage:
   - Updated dry-run script tests for matrix, bundle, full-suite, Makefile profile targets, and saturation presets to assert relay pump-batch forwarding visibility.

## Why

Relay pump-batch max is now a first-class runtime tuning knob. Without forwarding through saturation orchestrators, operators still have to patch scripts or run low-level commands manually. This slice keeps the tuning loop reproducible and configurable from top-level benchmark entrypoints.

## Validation

1. `bash -n benchmark-suite/scripts/run_lasm_cluster_saturation_boost_matrix.sh benchmark-suite/scripts/run_lasm_cluster_saturation_boost_bundle.sh benchmark-suite/scripts/run_full_benchmark_suite.sh benchmark-suite/scripts/test_run_lasm_cluster_saturation_boost_matrix.sh benchmark-suite/scripts/test_run_lasm_cluster_saturation_boost_bundle.sh benchmark-suite/scripts/test_run_full_benchmark_suite.sh benchmark-suite/scripts/test_makefile_profile_targets.sh benchmark-suite/scripts/test_bench_full_saturation_throughput_profile.sh benchmark-suite/scripts/test_bench_full_saturation_latency_profile.sh benchmark-suite/scripts/test_bench_full_saturation_presets_profile.sh`
2. `benchmark-suite/scripts/test_run_lasm_cluster_saturation_boost_matrix.sh`
3. `benchmark-suite/scripts/test_run_lasm_cluster_saturation_boost_bundle.sh`
4. `benchmark-suite/scripts/test_run_full_benchmark_suite.sh`
5. `benchmark-suite/scripts/test_makefile_profile_targets.sh`
6. `benchmark-suite/scripts/test_bench_full_saturation_throughput_profile.sh`
7. `benchmark-suite/scripts/test_bench_full_saturation_latency_profile.sh`
8. `benchmark-suite/scripts/test_bench_full_saturation_presets_profile.sh`
