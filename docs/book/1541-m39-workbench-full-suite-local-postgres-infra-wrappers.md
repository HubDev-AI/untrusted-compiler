# 1541 M39 Slice: Workbench Full-Suite Local Postgres Infra Wrappers

This slice adds local-infra wrappers for the workbench full-suite lanes so operators can run deterministic Postgres-backed runs without manually wiring DSN flags.

## What changed

1. Added local wrapper scripts:
   - `benchmark-suite/scripts/run_workbench_full_benchmark_suite_local.sh`
   - `benchmark-suite/scripts/run_workbench_full_benchmark_suite_local_repeats.sh`
2. Wrapper behavior:
   - orchestrates repo-local infra (`infra/local-postgres`) with `up/down/reset` control,
   - resolves DSN from `infra/local-postgres/.env` (fallback `.env.example` in dry-run mode),
   - injects:
     - `--lasm-db-adapter postgres`
     - `--lasm-postgres-dsn-file <temp-file>`
   - rejects conflicting passthrough flags for adapter/DSN overrides to keep local lane deterministic.
3. Added make targets:
   - `workbench-full-bench-local`
   - `workbench-full-bench-local-dry`
   - `workbench-full-bench-local-repeats`
   - `workbench-full-bench-local-repeats-dry`
4. Added script-level smoke tests:
   - `benchmark-suite/scripts/test_run_workbench_full_benchmark_suite_local.sh`
   - `benchmark-suite/scripts/test_run_workbench_full_benchmark_suite_local_repeats.sh`
5. Wired smoke tests into:
   - `benchmark-suite/Makefile` (`test-scripts`),
   - `.github/workflows/benchmark-smoke.yml`.

## Why

Workbench full-suite runs in Postgres mode previously required manual DSN plumbing and local infra lifecycle commands. That made repeated operator runs error-prone and inconsistent. The local wrappers mirror the proven alpha-postgres local orchestration pattern and provide one-command reproducible runs.

## Validation

1. `bash -n benchmark-suite/scripts/run_workbench_full_benchmark_suite_local.sh`
2. `bash -n benchmark-suite/scripts/run_workbench_full_benchmark_suite_local_repeats.sh`
3. `benchmark-suite/scripts/test_run_workbench_full_benchmark_suite_local.sh`
4. `benchmark-suite/scripts/test_run_workbench_full_benchmark_suite_local_repeats.sh`
5. `benchmark-suite/scripts/test_makefile_profile_targets.sh`
6. `make -C benchmark-suite workbench-full-bench-local-dry WORKBENCH_IMPLS=sec4 WORKBENCH_ENDPOINTS=wb-task-get WORKBENCH_REPEAT_RUNS=1`
