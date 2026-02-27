# 1543 M39 Slice: Workbench Matrix/Step Local Postgres Wrappers

This slice extends local-infra orchestration beyond full-suite lanes by adding wrappers for workbench matrix and step runners.

## What changed

1. Added local wrapper scripts:
   - `benchmark-suite/scripts/run_workbench_benchmark_matrix_local.sh`
   - `benchmark-suite/scripts/run_workbench_step_matrix_local.sh`
2. Wrapper behavior:
   - orchestrates repo-local Postgres infra (`infra/local-postgres`) with `up/down/reset`,
   - resolves DSN from local infra env defaults,
   - injects:
     - `--lasm-db-adapter postgres`
     - `--lasm-postgres-dsn-file <temp-file>`
   - rejects conflicting adapter/DSN passthrough flags for deterministic local operator behavior.
3. Added make targets:
   - `workbench-bench-local`
   - `workbench-bench-local-dry`
   - `workbench-step-bench-local`
   - `workbench-step-bench-local-dry`
4. Added smoke tests:
   - `benchmark-suite/scripts/test_run_workbench_benchmark_matrix_local.sh`
   - `benchmark-suite/scripts/test_run_workbench_step_matrix_local.sh`
5. Wired smoke tests into:
   - `benchmark-suite/Makefile` (`test-scripts`),
   - `.github/workflows/benchmark-smoke.yml`.

## Why

Local operators previously had infra wrappers only for full-suite flows. For shorter fixed-target or step-only loops, they still needed manual DSN wiring and infra lifecycle handling. These wrappers close that gap and make all workbench runner classes local-infra friendly.

## Validation

1. `bash -n benchmark-suite/scripts/run_workbench_benchmark_matrix_local.sh`
2. `bash -n benchmark-suite/scripts/run_workbench_step_matrix_local.sh`
3. `benchmark-suite/scripts/test_run_workbench_benchmark_matrix_local.sh`
4. `benchmark-suite/scripts/test_run_workbench_step_matrix_local.sh`
5. `benchmark-suite/scripts/test_makefile_profile_targets.sh`
6. `make -C benchmark-suite workbench-bench-local-dry WORKBENCH_IMPLS=sec4 WORKBENCH_ENDPOINTS=wb-task-get`
7. `make -C benchmark-suite workbench-step-bench-local-dry WORKBENCH_IMPLS=sec4 WORKBENCH_ENDPOINTS=wb-task-get`
