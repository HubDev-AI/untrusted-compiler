# 299 M10 Slice: Benchmark Smoke CI Gate

This chapter documents CI smoke coverage for core benchmark harness scripts.

## What it is

Added:
- `.github/workflows/benchmark-smoke.yml`

Updated:
- `docs/05-sec4-master-roadmap.md`

## Why it exists

M10 benchmark scripts define report and matrix contracts used by later analysis/publishing. Regressions in those scripts should be caught in pull requests, not after manual benchmark runs.

## How it works internally

The workflow runs on pull requests and `main` pushes:

1. Checkout repository.
2. Install stable Rust toolchain.
3. Install Node.js.
4. Execute benchmark smoke tests:
   - `benchmark-suite/scripts/test_preflight.sh`
   - `benchmark-suite/scripts/test_compare_matrix.sh`
   - `benchmark-suite/scripts/test_publish_report.sh`

These tests validate benchmark contract behavior without running full load tests.

## Inputs, outputs, and constraints

- Inputs:
  - benchmark script fixtures under `benchmark-suite/scripts/testdata`.
  - benchmark helper scripts under `benchmark-suite/scripts`.
- Output:
  - CI pass/fail signal for core benchmark script contracts.
- Constraint:
  - smoke gate does not execute `wrk2` load; it validates script/data contracts only.

## Failure modes and diagnostics

- Any benchmark script contract regression causes the workflow to fail.
- Script stderr includes direct mismatch context (missing keys, wrong leaders, missing sections, etc.).

## Example usage

Run locally:

```bash
benchmark-suite/scripts/test_preflight.sh
benchmark-suite/scripts/test_compare_matrix.sh
benchmark-suite/scripts/test_publish_report.sh
```

## Tradeoffs and next steps

- Tradeoff:
  - this gate does not benchmark runtime performance.
- Next:
  - optionally add a scheduled full benchmark job in controlled hardware environment.
