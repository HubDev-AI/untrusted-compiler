# 1510 M39 Slice: Alpha Postgres Repeats Markdown Report Renderer

## What changed

1. Added `benchmark-suite/scripts/render_alpha_postgres_comparison_suite_repeats_summary.sh`:
   - reads repeated-run summary JSON (`alpha-postgres-comparison-suite-repeats.json`),
   - emits markdown report with baseline/db-hot aggregate tables.
2. Report tables include:
   - samples count,
   - `requestsPerSec` mean/min/max,
   - `p99Ms` mean/min/max,
   - `rssKb` mean/min/max.
3. Added script test:
   - `benchmark-suite/scripts/test_render_alpha_postgres_comparison_suite_repeats_summary.sh`.
4. Added Make entrypoint:
   - `bench-alpha-postgres-suite-repeats-report`.

## Why

Repeated-run JSON is machine-friendly, but operators need a readable markdown artifact for quick review and sharing during alpha Postgres benchmark comparison loops.

## Validation

1. `benchmark-suite/scripts/test_render_alpha_postgres_comparison_suite_repeats_summary.sh`
2. `benchmark-suite/scripts/test_run_alpha_postgres_comparison_suite_repeats.sh`
3. `make -C benchmark-suite bench-alpha-postgres-suite-repeats-report ALPHA_POSTGRES_REPEATS_SUMMARY=<temp_summary> ALPHA_POSTGRES_REPEATS_REPORT=<temp_report>`
