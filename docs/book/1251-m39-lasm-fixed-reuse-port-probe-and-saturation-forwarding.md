# 1251 M39 Slice: LASM Fixed Reuse-Port Probe + Saturation Forwarding

This slice adds explicit fixed reuse-port mode support to the LASM capacity probe and forwards that mode through saturation/full-suite orchestration.

## What changed

1. Capacity probe mode:
   - `run_lasm_cluster_capacity_probe.sh` now supports `--fixed-reuse-port-mode`.
   - In fixed mode, probe forces `autoscaleMaxInstances=instances` to activate fixed reuse-port cluster execution.
2. Fixed-mode guardrails:
   - Probe rejects relay-proxy-only tuning flags in fixed mode:
     - `--cluster-relay-workers`
     - `--cluster-relay-queue`
     - `--cluster-accept-workers`
     - `--cluster-relay-accept-batch-max`
     - `--cluster-relay-pump-batch-max`
   - Probe rejects `--keep-cluster-status-json` in fixed mode.
   - Probe plan output now explicitly marks:
     - `fixedReusePortMode=true|false`
     - `clusterStatusJson=n/a (fixed-reuse-port-mode)` when fixed mode is active.
3. Probe JSON artifact:
   - `run.fixedReusePortMode` is now persisted in probe summary output.
   - `artifacts.clusterStatusJson` becomes `null` when fixed mode is active.
4. Saturation/full-suite forwarding:
   - `run_lasm_cluster_saturation_boost_matrix.sh` now accepts/passes `--fixed-reuse-port-mode` and records `run.fixedReusePortMode`.
   - `run_lasm_cluster_saturation_boost_bundle.sh` now accepts/passes `--fixed-reuse-port-mode`.
   - `run_full_benchmark_suite.sh` now accepts `--saturation-fixed-reuse-port-mode` and forwards it into the saturation lane.
   - `benchmark-suite/Makefile` now includes fixed-mode forwarding variables/flags for full saturation targets and direct LASM probe/matrix/bundle targets.
5. Contract coverage:
   - Updated probe/matrix/bundle/full-suite/Makefile dry-run tests to cover fixed-mode propagation and fixed-mode guard diagnostics.

## Why

Fixed reuse-port mode uses a different runtime path than proxy-relay mode and should be benchmarked directly. Without first-class orchestration support, this required manual command rewrites and broke deterministic capacity workflows.

## Sample result

Short local sample (`20s`, `8t/256c`, `/health`) in this environment:

- proxy-relay cluster mode: `~77.6k req/s`, `p99 ~5.47ms`
- fixed reuse-port mode: `~120.5k req/s`, `p99 ~13.25ms`

This shows fixed mode materially improves throughput in this environment while changing latency profile, so it is now a first-class tuning lane.

## Validation

1. `bash -n benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh benchmark-suite/scripts/test_run_lasm_cluster_capacity_probe.sh benchmark-suite/scripts/run_lasm_cluster_saturation_boost_matrix.sh benchmark-suite/scripts/run_lasm_cluster_saturation_boost_bundle.sh benchmark-suite/scripts/run_full_benchmark_suite.sh benchmark-suite/scripts/test_run_lasm_cluster_saturation_boost_matrix.sh benchmark-suite/scripts/test_run_lasm_cluster_saturation_boost_bundle.sh benchmark-suite/scripts/test_run_full_benchmark_suite.sh benchmark-suite/scripts/test_makefile_profile_targets.sh`
2. `benchmark-suite/scripts/test_run_lasm_cluster_capacity_probe.sh`
3. `benchmark-suite/scripts/test_run_lasm_cluster_saturation_boost_matrix.sh`
4. `benchmark-suite/scripts/test_run_lasm_cluster_saturation_boost_bundle.sh`
5. `benchmark-suite/scripts/test_run_full_benchmark_suite.sh`
6. `benchmark-suite/scripts/test_makefile_profile_targets.sh`
7. `benchmark-suite/scripts/test_bench_full_saturation_throughput_profile.sh`
8. `benchmark-suite/scripts/test_bench_full_saturation_latency_profile.sh`
9. `benchmark-suite/scripts/test_bench_full_saturation_presets_profile.sh`
