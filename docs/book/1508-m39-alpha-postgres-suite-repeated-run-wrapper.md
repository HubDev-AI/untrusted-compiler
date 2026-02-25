# 1508 M39 Slice: Alpha Postgres Suite Repeated-Run Wrapper

## What changed

1. Added `benchmark-suite/scripts/run_alpha_postgres_comparison_suite_repeats.sh`:
   - runs `run_alpha_postgres_comparison_suite.sh` repeatedly with identical forwarded args,
   - supports wrapper options: `--runs <n>`, `--dry-run`, `--out <path>`.
2. Added run-scoped artifact snapshot handling for non-dry runs:
   - copies baseline/db-hot artifacts per run under `results/summaries/alpha-postgres-comparison-suite-runs/`,
   - rewrites each per-run summary to point to run-scoped artifacts.
3. Added aggregate repeated-run summary output:
   - default: `results/summaries/alpha-postgres-comparison-suite-repeats.json`,
   - includes run count, forwarded args, and per-run summary/matrix references.
4. Added script test:
   - `benchmark-suite/scripts/test_run_alpha_postgres_comparison_suite_repeats.sh`.
5. Added Make entrypoints:
   - `bench-alpha-postgres-suite-repeats`,
   - `bench-alpha-postgres-suite-repeats-dry`,
   - `BENCH_ALPHA_POSTGRES_REPEAT_RUNS` (default `3`).

## Why

Operators need deterministic repeated runs under the same configuration to compare stability/regression trends without manually re-running and renaming artifacts after each suite pass.

## Validation

1. `benchmark-suite/scripts/test_run_alpha_postgres_comparison_suite_repeats.sh`
2. `benchmark-suite/scripts/test_run_alpha_postgres_comparison_suite.sh`
3. `make -C benchmark-suite bench-alpha-postgres-suite-repeats-dry BENCH_LASM_DB_POSTGRES_DSN_FILE=/tmp/postgres.dsn BENCH_ALPHA_POSTGRES_REPEAT_RUNS=2`
