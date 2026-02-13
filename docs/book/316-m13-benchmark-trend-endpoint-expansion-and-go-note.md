# 316 M13 Slice: Benchmark Trend Endpoint Expansion and Go Note

This chapter documents expanding scoped live benchmark trend checks from one endpoint to two and recording M13-S1 go status.

## What it is

Updated:
- `.github/workflows/benchmark-trend.yml`
- `benchmark-suite/baselines/node-decode-trend-baseline.json`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/312-m13-scoped-live-benchmark-workflow-and-threshold-guard.md`

## Why it exists

After initial scoped stability checks on `ping`, M13-S1 required one additional endpoint for broader signal before closing the first operational-confidence slice.

## How it works internally

- Scheduled trend workflow now runs:
  - `run_full_benchmark_suite.sh --impls node --endpoints ping,decode`
- Threshold checks now run per endpoint:
  - `ping` with ping baseline,
  - `decode` with decode baseline.
- Added decode baseline policy file:
  - `benchmark-suite/baselines/node-decode-trend-baseline.json`.

## Roadmap state updates

- M13-S1 tracking checklist now has all items completed.
- Added explicit M13-S1 go/no-go note:
  - current status `GO`,
  - revisit trigger conditions documented for threshold drift/verifier issues.

## Inputs, outputs, and constraints

- Inputs:
  - compare-matrix output from scheduled live run,
  - endpoint-specific baseline policy files.
- Output:
  - broader trend signal with deterministic endpoint-specific pass/fail checks.
- Constraint:
  - still limited to one implementation (`node`) for cost control.

## Example workflow lines

```bash
benchmark-suite/scripts/run_full_benchmark_suite.sh --impls node --endpoints ping,decode
benchmark-suite/scripts/check_regression_thresholds.sh ... --endpoint ping --baseline benchmark-suite/baselines/node-ping-trend-baseline.json
benchmark-suite/scripts/check_regression_thresholds.sh ... --endpoint decode --baseline benchmark-suite/baselines/node-decode-trend-baseline.json
```

## Tradeoffs and next steps

- Tradeoff:
  - wider endpoint scope increases runtime/variability slightly.
- Next:
  - tune decode thresholds from scheduled run data and define M13-S2 candidate scope.
