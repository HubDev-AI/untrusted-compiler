# 1503 M39 Slice: Alpha Postgres Suite Run-Fingerprint Metadata

## What changed

1. Extended `benchmark-suite/scripts/run_alpha_postgres_comparison_suite.sh` summary payload to include `runContext` metadata:
   - `repoRevision` (short git revision)
   - `dryRun`
   - DSN source mode (`file` vs `env`) and file path when provided
   - host fingerprint (`uname`, logical CPU count)
   - active benchmark override environment values (`BENCH_THREADS`, `BENCH_CONNECTIONS`, `BENCH_DURATION`, `BENCH_TARGET`, `BENCH_PORT`, `BENCH_STEP_RATES`, `BENCH_STEP_DURATION`)
2. Kept artifact snapshot outputs unchanged (`*-alpha-base.*`, `*-alpha-db-postgres.*`).
3. Updated benchmark suite README notes to document the new summary metadata.

## Why

When comparing runs across days/branches, operators need the exact load/runtime conditions captured with the artifact bundle. Embedding run fingerprint metadata in the suite summary makes same-condition verification explicit.

## Validation

1. `benchmark-suite/scripts/test_run_alpha_postgres_comparison_suite.sh`
2. `benchmark-suite/scripts/test_run_alpha_postgres_comparison_suite_local.sh`
3. `make -C benchmark-suite bench-alpha-postgres-suite-local-dry`
