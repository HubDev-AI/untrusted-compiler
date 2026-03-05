# 1509 M39 Slice: Alpha Postgres Suite Local Repeated-Run Wrapper

## What changed

1. Added `benchmark-suite/scripts/run_alpha_postgres_comparison_suite_local_repeats.sh`:
   - orchestrates local Postgres infra (`infra/local-postgres`) and delegates to `run_alpha_postgres_comparison_suite_repeats.sh`,
   - auto-generates DSN temp file from local infra env (same DSN resolution model as local single-run wrapper).
2. Added local controls:
   - `--dry-run`,
   - `--keep-up`,
   - `--reset-db`,
   - `--infra-env`.
3. Added Make entrypoints:
   - `bench-alpha-postgres-suite-local-repeats`,
   - `bench-alpha-postgres-suite-local-repeats-dry`.
4. Added script test:
   - `benchmark-suite/scripts/test_run_alpha_postgres_comparison_suite_local_repeats.sh`.

## Why

Repeated cross-language Postgres comparison runs should be one command in local development too, without manually managing Docker infra lifecycle and DSN wiring.

## Validation

1. `benchmark-suite/scripts/test_run_alpha_postgres_comparison_suite_local_repeats.sh`
2. `benchmark-suite/scripts/test_run_alpha_postgres_comparison_suite_repeats.sh`
3. `make -C benchmark-suite bench-alpha-postgres-suite-local-repeats-dry BENCH_ALPHA_POSTGRES_REPEAT_RUNS=2`
