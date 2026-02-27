# 1540 M39 Slice: Workbench Full-Suite CI Smoke Gates

This slice adds benchmark-smoke CI coverage for the new workbench full-suite runners and renderer.

## What changed

1. Added script-level dry-run contract tests:
   - `benchmark-suite/scripts/test_run_workbench_full_benchmark_suite.sh`
   - `benchmark-suite/scripts/test_run_workbench_full_benchmark_suite_repeats.sh`
   - `benchmark-suite/scripts/test_render_workbench_full_benchmark_suite_repeats_summary.sh`
2. Expanded makefile target contract checks:
   - `benchmark-suite/scripts/test_makefile_profile_targets.sh` now asserts workbench full-suite target wiring (`workbench-full-bench*`, repeats, and repeats-report).
3. Wired CI smoke execution:
   - added new tests to `.github/workflows/benchmark-smoke.yml`.
4. Wired local smoke bundle execution:
   - added new tests to `benchmark-suite/Makefile` under `test-scripts`.

## Why

Workbench full-suite orchestration had landed quickly across multiple slices. Without smoke-gate coverage, regressions in dry-run command composition or target wiring could silently break benchmark operator workflows. These tests lock expected behavior and keep CI feedback immediate.

## Validation

1. `benchmark-suite/scripts/test_run_workbench_full_benchmark_suite.sh`
2. `benchmark-suite/scripts/test_run_workbench_full_benchmark_suite_repeats.sh`
3. `benchmark-suite/scripts/test_render_workbench_full_benchmark_suite_repeats_summary.sh`
4. `benchmark-suite/scripts/test_makefile_profile_targets.sh`
