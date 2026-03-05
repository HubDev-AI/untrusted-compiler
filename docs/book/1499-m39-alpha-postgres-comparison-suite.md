# M39 Slice: Alpha Postgres Comparison Suite

Date: 2026-02-26  
Milestone: M39 (benchmark execution lane)

## What changed

- Added `benchmark-suite/scripts/run_alpha_postgres_comparison_suite.sh`.
- The script runs two deterministic phases in one command:
  1. baseline endpoints (`ping`, `decode`, `users-post`, `users-get`) in LASM Postgres mode.
  2. DB hot-path endpoints (`db-hot-write`, `db-hot-write-tx`, `db-hot-query-one`, `db-records`) in LASM Postgres mode.
- Added deterministic artifact snapshotting per phase:
  - baseline snapshots: `*-alpha-base.*`
  - db-hot snapshots: `*-alpha-db-postgres.*`
- Added summary output:
  - `benchmark-suite/results/summaries/alpha-postgres-comparison-suite.json`
- Added make wrappers:
  - `bench-alpha-postgres-suite`
  - `bench-alpha-postgres-suite-dry`

## Why

- Operators needed a single repeatable entrypoint for “same conditions every run” Postgres-mode comparisons instead of manually chaining multiple matrix/full commands.
- Snapshotting avoids artifact overwrite between baseline and DB-hot phases and makes trend ingestion deterministic.

## Behavioral contract

- Suite requires Postgres DSN source:
  - `--lasm-db-postgres-dsn-file`, or
  - `SEC4_DB_ALPHA_DB_POSTGRES_DSN` or `SEC4_RT_LASM_DB_POSTGRES_DSN`.
- `--dry-run` executes both delegated plans and prints snapshot actions without mutating benchmark artifacts.
- Non-dry runs execute both phases sequentially and write deterministic snapshot artifact paths plus summary JSON.

## Validation

- `benchmark-suite/scripts/test_run_alpha_postgres_comparison_suite.sh`
- `benchmark-suite/scripts/test_makefile_profile_targets.sh`
