# 308 M10 Slice: Benchmark CI Dry-Run Orchestrator Gates

This chapter documents expanding benchmark CI with deterministic dry-run orchestrator checks.

## What it is

Updated:
- `.github/workflows/benchmark-smoke.yml`
- `docs/05-sec4-master-roadmap.md`

## Why it exists

Benchmark smoke CI previously covered only a subset of scripts. This slice adds dry-run orchestration checks so matrix/full-suite contract drift is caught earlier without needing live benchmark services in CI.

## How it works internally

`benchmark-smoke.yml` now runs additional benchmark script tests:
- `benchmark-suite/scripts/test_run_comparison_matrix.sh`
- `benchmark-suite/scripts/test_run_step_matrix.sh`
- `benchmark-suite/scripts/test_run_full_benchmark_suite.sh`

These tests validate dry-run command composition and scoped orchestrator behavior for:
- fixed-target matrix flow,
- step-load matrix flow,
- full-suite combined flow.

## Inputs, outputs, and constraints

- Inputs:
  - benchmark orchestration scripts and test fixtures.
- Output:
  - CI pass/fail signal on dry-run orchestrator contract validity.
- Constraint:
  - checks remain dry-run only; they validate orchestration contracts, not runtime benchmark performance.

## Example usage

```bash
benchmark-suite/scripts/test_run_comparison_matrix.sh
benchmark-suite/scripts/test_run_step_matrix.sh
benchmark-suite/scripts/test_run_full_benchmark_suite.sh
```

## Tradeoffs and next steps

- Tradeoff:
  - dry-run tests do not validate service startup or workload execution.
- Next:
  - add optional scheduled CI workflow for scoped live benchmark execution and artifact verification.
