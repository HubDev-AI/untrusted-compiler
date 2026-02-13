# 312 M13 Slice: Scoped Live Benchmark Workflow and Threshold Guard

This chapter documents the first executable M13 slice for scheduled trend checks.

## What it is

Updated:
- `.github/workflows/benchmark-trend.yml`
- `benchmark-suite/scripts/check_regression_thresholds.sh`
- `benchmark-suite/scripts/test_check_regression_thresholds.sh`
- `benchmark-suite/baselines/node-ping-trend-baseline.json`
- `benchmark-suite/baselines/node-decode-trend-baseline.json`
- `.github/workflows/benchmark-smoke.yml`
- `docs/05-sec4-master-roadmap.md`

## Why it exists

Dry-run benchmark CI catches orchestration drift, but M13 requires lightweight live execution and deterministic regression signaling over real benchmark artifacts.

## How it works internally

### 1) Scheduled scoped live run

`benchmark-trend.yml` runs on:
- weekly schedule,
- manual dispatch.

Workflow behavior:
1. installs Node + `wrk2`,
2. runs `run_full_benchmark_suite.sh --impls node --endpoints ping`,
3. runs baseline-aware threshold checks for `ping` and `decode`,
4. uploads resulting benchmark artifacts with retention policy (30 days).

### 2) Regression threshold guard

`check_regression_thresholds.sh` evaluates leader metrics from `compare-matrix.json` for a target endpoint:
- `p99` must be <= configured max,
- target coverage (`requestsPerSec / targetRps`) must be >= configured minimum.
- optional baseline policy file enforces relative regression limits against baseline metrics.

If either threshold is violated, command fails with a deterministic message.

### 3) Smoke coverage

`benchmark-smoke.yml` now executes:
- `test_check_regression_thresholds.sh`

This keeps threshold-check logic verified in normal PR/push CI.

## Inputs, outputs, and constraints

- Inputs:
  - `benchmark-suite/results/summaries/compare-matrix.json`
  - `benchmark-suite/baselines/node-ping-trend-baseline.json` (scheduled workflow baseline policy)
  - endpoint + threshold parameters.
- Output:
  - pass/fail regression gate signal.
- Constraint:
  - current workflow scope is intentionally narrow (`node + ping + decode`) for cost/runtime control.

## Example usage

```bash
benchmark-suite/scripts/check_regression_thresholds.sh \
  benchmark-suite/results/summaries/compare-matrix.json \
  --endpoint ping \
  --max-p99-ms 30 \
  --min-target-coverage 85 \
  --baseline benchmark-suite/baselines/node-ping-trend-baseline.json

benchmark-suite/scripts/check_regression_thresholds.sh \
  benchmark-suite/results/summaries/compare-matrix.json \
  --endpoint decode \
  --max-p99-ms 80 \
  --min-target-coverage 60 \
  --baseline benchmark-suite/baselines/node-decode-trend-baseline.json
```

## Tradeoffs and next steps

- Tradeoff:
  - one impl/endpoint scope is low cost but limited signal.
- Next:
  - expand scope incrementally (additional endpoint/impl) once trend stability is confirmed.
