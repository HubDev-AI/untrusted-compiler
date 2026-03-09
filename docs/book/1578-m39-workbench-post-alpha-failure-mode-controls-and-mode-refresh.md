# M39: Workbench Post-Alpha Failure-Mode Controls and Mode Refresh

Date: 2026-03-09  
Milestone: M39 post-alpha tuning

## What was implemented

1. Added deterministic failure-mode control for cross-runtime workbench orchestration:
   - `benchmark-suite/scripts/run_workbench_benchmark_matrix.sh`
   - `benchmark-suite/scripts/run_workbench_step_matrix.sh`
   - `benchmark-suite/scripts/run_workbench_full_benchmark_suite.sh` (forwards the flag)
2. New switch:
   - `--fail-on-impl-failure 0|1` (default `1`)
3. Added profile retry control in matrix runner:
   - `--profile-retry-on-failure <n>` (default `1`)

## Why this was needed

Post-alpha tuning and publication loops were being blocked by non-sec4 competitor-lane instability.  
The strict all-or-nothing exit behavior prevented artifact generation even when sec4-lasm itself was healthy.

The new controls keep strict behavior as default for gates, but allow deterministic relaxed-mode publication/tuning runs that still preserve failed-lane accounting in run summaries.

## Runtime/benchmark evidence refresh

After the new controls, LASM mode-compare was rerun on canonical Postgres workload and produced a refreshed recommendation artifact:

- `benchmark-suite/results/summaries/workbench-lasm-mode-compare-repeats.json`
- recommendation: `mode=proxy`
- reason: `medianRequestsPerSec=5362.34`

## Validation

Focused script contract checks:

- `benchmark-suite/scripts/test_run_workbench_benchmark_matrix.sh`
- `benchmark-suite/scripts/test_run_workbench_step_matrix.sh`
- `benchmark-suite/scripts/test_run_workbench_full_benchmark_suite.sh`

All passed.
