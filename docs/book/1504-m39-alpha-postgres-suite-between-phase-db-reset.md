# 1504 M39 Slice: Alpha Postgres Suite Between-Phase DB Reset

## What changed

1. Added `--reset-db-between-phases` to `benchmark-suite/scripts/run_alpha_postgres_comparison_suite.sh`.
2. When enabled, the suite now drops benchmark tables before DB-hot phase execution:
   - `sec4_lasm_db_records`
   - `bench_users`
   - `users`
3. Dry-run mode prints deterministic reset markers without requiring `psql` execution.
4. Updated alpha Postgres suite script coverage to assert dry-run reset markers.

## Why

Repeated Postgres comparison runs can accumulate benchmark artifacts and affect DB-hot measurements. The optional between-phase reset keeps DB-hot phase starts clean while preserving baseline phase flow.

## Validation

1. `benchmark-suite/scripts/test_run_alpha_postgres_comparison_suite.sh`
2. `benchmark-suite/scripts/test_run_alpha_postgres_comparison_suite_local.sh`
3. `make -C benchmark-suite bench-alpha-postgres-suite-local-dry`
