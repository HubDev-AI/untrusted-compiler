# 1500 M39 Slice: Alpha Postgres Comparison Suite Local Infra Wrapper

## What changed

1. Added `benchmark-suite/scripts/run_alpha_postgres_comparison_suite_local.sh`.
   - Wraps `run_alpha_postgres_comparison_suite.sh` with repo-local Postgres infra orchestration (`infra/local-postgres`).
   - Supports lifecycle options:
     - `--reset-db` (recreate local data before run),
     - `--keep-up` (keep infra running after run),
     - `--dry-run` (no docker start/stop; delegated plan only).
   - Auto-resolves DSN from infra env (`.env` or `.env.example`) and passes it via temporary `--lasm-db-postgres-dsn-file`.
2. Added Make targets:
   - `bench-alpha-postgres-suite-local`
   - `bench-alpha-postgres-suite-local-dry`
3. Added script coverage:
   - `benchmark-suite/scripts/test_run_alpha_postgres_comparison_suite_local.sh`
4. Updated benchmark suite README with local one-command Postgres suite usage.

## Why

Postgres-mode comparison loops required manual infra startup + manual DSN wiring before running the alpha suite. This wrapper makes local same-condition runs one command and keeps the delegated suite contract unchanged.

## Validation

1. `benchmark-suite/scripts/test_run_alpha_postgres_comparison_suite_local.sh`
2. `benchmark-suite/scripts/test_run_alpha_postgres_comparison_suite.sh`
3. `make -C benchmark-suite bench-alpha-postgres-suite-local-dry`
