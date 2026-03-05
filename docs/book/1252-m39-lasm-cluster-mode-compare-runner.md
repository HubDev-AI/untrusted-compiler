# 1252 M39 Slice: LASM Cluster Mode Compare Runner

This slice adds a deterministic comparison runner that benchmarks proxy-relay mode vs fixed reuse-port mode under one shared workload profile.

## What changed

1. New comparison script:
   - `benchmark-suite/scripts/run_lasm_cluster_mode_compare.sh`
   - Runs two probes sequentially with shared profile inputs:
     - proxy-relay (`run_lasm_cluster_capacity_probe.sh`)
     - fixed reuse-port (`run_lasm_cluster_capacity_probe.sh --fixed-reuse-port-mode`)
2. Deterministic comparison artifact:
   - emits one JSON summary containing:
     - `comparison.recommendedMode`
     - `comparison.requestsPerSecDelta`
     - `comparison.requestsPerSecGainPctVsProxy`
     - `comparison.p99ProxyMs` / `comparison.p99FixedMs`
     - `comparison.peakRssDeltaKb`
   - preserves the original proxy/fixed probe payloads inside the artifact for traceability.
3. Makefile integration:
   - Added `lasm-cluster-mode-compare` target in `benchmark-suite/Makefile`.
   - Added target to benchmark help output and `test-scripts` execution list.
4. Contract test:
   - Added `benchmark-suite/scripts/test_run_lasm_cluster_mode_compare.sh` with:
     - dry-run plan coverage,
     - delegated fixed-mode marker coverage,
     - invalid input diagnostics.

## Why

Proxy-relay and fixed reuse-port are materially different runtime paths. Tuning decisions were previously based on ad-hoc manual probe runs. This runner makes mode selection reproducible and comparable in one artifact for every workload profile.

## Sample result

Short smoke sample (`10s`, `4t/128c`, `/health`) in this environment:

- proxy-relay: `~71.2k req/s`
- fixed reuse-port: `~128.6k req/s`
- computed gain: `~80.7%`
- recommended mode: `fixed-reuse-port`

## Validation

1. `bash -n benchmark-suite/scripts/run_lasm_cluster_mode_compare.sh benchmark-suite/scripts/test_run_lasm_cluster_mode_compare.sh benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh benchmark-suite/scripts/run_lasm_cluster_saturation_boost_matrix.sh benchmark-suite/scripts/run_lasm_cluster_saturation_boost_bundle.sh benchmark-suite/scripts/run_full_benchmark_suite.sh benchmark-suite/scripts/test_makefile_profile_targets.sh`
2. `benchmark-suite/scripts/test_run_lasm_cluster_mode_compare.sh`
3. `benchmark-suite/scripts/test_run_lasm_cluster_capacity_probe.sh`
4. `benchmark-suite/scripts/test_run_lasm_cluster_saturation_boost_matrix.sh`
5. `benchmark-suite/scripts/test_run_lasm_cluster_saturation_boost_bundle.sh`
6. `benchmark-suite/scripts/test_run_full_benchmark_suite.sh`
7. `benchmark-suite/scripts/test_makefile_profile_targets.sh`
8. `benchmark-suite/scripts/run_lasm_cluster_mode_compare.sh --skip-build --duration 10s --threads 4 --connections 128 --target-requests 300000 --out results/summaries/sec4-lasm-cluster-mode-compare-smoke.json --proxy-out results/summaries/sec4-lasm-cluster-mode-compare-smoke-proxy.json --fixed-out results/summaries/sec4-lasm-cluster-mode-compare-smoke-fixed.json`
