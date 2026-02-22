# M39: LASM Mode-Compare Build-Profile + Multi-Sample Wiring

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `benchmark-suite/scripts/run_lasm_cluster_mode_compare.sh`:
  - added `--build-profile <debug|release>` (default `release`),
  - added `--samples <n>` (default `1`),
  - forwards both controls to proxy and fixed probe runs (`run_lasm_cluster_capacity_probe.sh`),
  - validates invalid `build-profile` and `samples` inputs before probe execution.
- Updated `benchmark-suite/scripts/test_run_lasm_cluster_mode_compare.sh`:
  - dry-run now asserts `buildProfile` + `samples` markers,
  - added deterministic negative checks for invalid `--samples` and invalid `--build-profile`.

## Why

Mode comparisons were still using implicit single-sample defaults. After adding profile/sample support in the base probe script, compare orchestration needed the same controls so proxy-vs-fixed recommendations are computed from consistent run settings.

## Result

- One command now drives stable proxy/fixed comparisons with explicit probe profile and sample count:
  - `run_lasm_cluster_mode_compare.sh --build-profile release --samples 3 ...`
- Compare outputs preserve forwarded run settings in both probe artifacts, which keeps recommendation context explicit.

## Validation

- `bash -n benchmark-suite/scripts/run_lasm_cluster_mode_compare.sh`
- `bash -n benchmark-suite/scripts/test_run_lasm_cluster_mode_compare.sh`
- `benchmark-suite/scripts/test_run_lasm_cluster_mode_compare.sh`
- `benchmark-suite/scripts/run_lasm_cluster_mode_compare.sh --duration 4s --threads 4 --connections 64 --target-requests 1 --instances 2 --autoscale-max-instances 3 --build-profile debug --samples 2 --skip-build --proxy-out results/summaries/sec4-lasm-mode-compare-smoke-proxy.json --fixed-out results/summaries/sec4-lasm-mode-compare-smoke-fixed.json --out results/summaries/sec4-lasm-mode-compare-smoke.json`
