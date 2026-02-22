# M39: LASM Benchmark Wrapper wrk-Process Passthrough

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated LASM benchmark wrappers to accept/forward wrk-process fanout:
  - `benchmark-suite/scripts/run_lasm_cluster_mode_compare.sh`
  - `benchmark-suite/scripts/run_lasm_cluster_saturation_boost_matrix.sh`
  - `benchmark-suite/scripts/run_lasm_cluster_saturation_boost_bundle.sh`
- Added top-level full-suite passthrough:
  - `benchmark-suite/scripts/run_full_benchmark_suite.sh` now accepts
    `--saturation-wrk-processes <n>` and forwards it into both:
    - LASM saturation tuning bundle phase,
    - LASM mode-compare phase.
- Updated script tests:
  - `test_run_lasm_cluster_mode_compare.sh`
  - `test_run_lasm_cluster_saturation_boost_matrix.sh`
  - `test_run_lasm_cluster_saturation_boost_bundle.sh`
  - `test_run_full_benchmark_suite.sh`
  - each now validates `wrkProcesses` dry-run passthrough and deterministic
    invalid-input diagnostics (`integer`, `>= 1`) where applicable.

## Why

The capacity probe already supported multi-process wrk fanout, but wrapper layers still capped users at single-process loadgen unless they called the probe directly. This slice removes that gap so full benchmark flows can scale client-side pressure consistently.

## Result

- `wrk` fanout can now be configured from every LASM benchmark entrypoint.
- Saturation matrix summaries include run-level wrk fanout metadata and per-run passthrough remains deterministic.
- Full-suite saturation/mode-compare runs can share one operator-level knob (`--saturation-wrk-processes`) without manual script rewiring.

## Validation

- `bash -n benchmark-suite/scripts/run_lasm_cluster_mode_compare.sh`
- `bash -n benchmark-suite/scripts/test_run_lasm_cluster_mode_compare.sh`
- `bash -n benchmark-suite/scripts/run_lasm_cluster_saturation_boost_matrix.sh`
- `bash -n benchmark-suite/scripts/test_run_lasm_cluster_saturation_boost_matrix.sh`
- `bash -n benchmark-suite/scripts/run_lasm_cluster_saturation_boost_bundle.sh`
- `bash -n benchmark-suite/scripts/test_run_lasm_cluster_saturation_boost_bundle.sh`
- `bash -n benchmark-suite/scripts/run_full_benchmark_suite.sh`
- `bash -n benchmark-suite/scripts/test_run_full_benchmark_suite.sh`
- `benchmark-suite/scripts/test_run_lasm_cluster_mode_compare.sh`
- `benchmark-suite/scripts/test_run_lasm_cluster_saturation_boost_matrix.sh`
- `benchmark-suite/scripts/test_run_lasm_cluster_saturation_boost_bundle.sh`
- `benchmark-suite/scripts/test_run_full_benchmark_suite.sh`
