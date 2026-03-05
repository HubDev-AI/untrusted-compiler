# 1253 M39 Slice: LASM Full-Suite Optional Mode-Compare Lane

This slice wires the LASM proxy-vs-fixed mode comparison runner into the full benchmark suite so operators can produce mode-comparison artifacts from one top-level run.

## What changed

1. Full-suite runner flag:
   - `benchmark-suite/scripts/run_full_benchmark_suite.sh` now accepts `--include-lasm-mode-compare`.
   - When enabled, it runs a dedicated `phase: lasm mode compare` using `run_lasm_cluster_mode_compare.sh`.
2. Deterministic guardrail:
   - `--include-lasm-mode-compare` now requires `sec4-lasm` in `--impls`.
   - Invalid combinations fail fast with deterministic diagnostics.
3. Shared tuning-forwarding:
   - Existing LASM saturation tuning inputs are forwarded into mode compare:
     - `--saturation-duration`
     - `--saturation-threads`
     - `--saturation-connections`
     - `--saturation-target-requests`
     - `--saturation-cluster-relay-workers`
     - `--saturation-cluster-relay-queue`
     - `--saturation-cluster-accept-workers`
     - `--saturation-cluster-relay-accept-batch-max`
     - `--saturation-cluster-relay-pump-batch-max`
4. Makefile forwarding:
   - Added `FULL_LASM_INCLUDE_MODE_COMPARE` (`false` by default) and `LASM_MODE_COMPARE_FLAG`.
   - `bench-full`, `bench-full-dry`, `bench-full-saturation`, and `bench-full-saturation-dry` now forward the optional mode-compare lane toggle.
5. Script contract coverage:
   - `benchmark-suite/scripts/test_run_full_benchmark_suite.sh` now checks mode-compare phase delegation, tuning forwarding, and invalid-impl guard behavior.
   - `benchmark-suite/scripts/test_makefile_profile_targets.sh` now checks that full-suite targets include the mode-compare forwarding token.

## Why

Mode comparison should be a first-class output of the same orchestration path operators already run for benchmark bundles. This avoids ad-hoc extra commands and keeps throughput/latency mode decisions reproducible from a single suite invocation.

## Validation

1. `bash -n benchmark-suite/scripts/run_full_benchmark_suite.sh benchmark-suite/scripts/test_run_full_benchmark_suite.sh benchmark-suite/Makefile benchmark-suite/scripts/test_makefile_profile_targets.sh`
2. `benchmark-suite/scripts/test_run_full_benchmark_suite.sh`
3. `benchmark-suite/scripts/test_makefile_profile_targets.sh`
