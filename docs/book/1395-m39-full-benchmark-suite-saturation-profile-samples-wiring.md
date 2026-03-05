# M39: Full Benchmark Suite Saturation Profile/Samples Wiring

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `benchmark-suite/scripts/run_full_benchmark_suite.sh`:
  - added top-level flags:
    - `--saturation-build-profile <debug|release>`
    - `--saturation-samples <n>`
  - forwards these controls into:
    - LASM saturation bundle phase (`run_lasm_cluster_saturation_boost_bundle.sh`),
    - LASM mode-compare phase (`run_lasm_cluster_mode_compare.sh`).
- Updated `benchmark-suite/scripts/test_run_full_benchmark_suite.sh`:
  - tuned saturation dry-run fixture now verifies delegated `buildProfile` and `samples`,
  - mode-compare dry-run fixture now verifies delegated `buildProfile` and `samples`.

## Why

After stabilizing profile/sample controls in the probe and orchestration wrappers, the top-level benchmark suite still lacked a direct way to set them. This created drift between standalone LASM tuning runs and full-suite orchestrated runs.

## Result

- One entrypoint now controls probe profile/sample behavior end-to-end:
  - `run_full_benchmark_suite.sh --include-lasm-saturation --saturation-build-profile release --saturation-samples 3 ...`
- Full-suite dry-run tests now lock this passthrough contract.

## Validation

- `bash -n benchmark-suite/scripts/run_full_benchmark_suite.sh`
- `bash -n benchmark-suite/scripts/test_run_full_benchmark_suite.sh`
- `benchmark-suite/scripts/test_run_full_benchmark_suite.sh`
