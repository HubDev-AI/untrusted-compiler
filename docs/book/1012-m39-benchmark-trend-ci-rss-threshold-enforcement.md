# 1012 M39 Slice: Benchmark Trend CI RSS Threshold Enforcement

## What It Is

This slice wires RSS threshold usage into scheduled benchmark trend CI:

- `.github/workflows/benchmark-trend.yml` now passes `--max-rss-kb` in both ping/decode regression-threshold steps.
- trend workflow contract tests now require RSS threshold flag presence.

## Why It Exists

After adding RSS metrics and guard support, trend CI still enforced only latency/coverage thresholds in workflow wiring. This slice closes that gap so memory thresholds are exercised in automation.

## How It Works Internally

1. Workflow changes:
   - ping threshold step includes `--max-rss-kb 500000`.
   - decode threshold step includes `--max-rss-kb 500000`.

2. Contract hardening:
   - `scripts/test-benchmark-trend-workflow-contract.sh` now requires `--max-rss-kb` token in workflow content.
   - `scripts/test-benchmark-trend-workflow-contract-guard.sh` includes a negative fixture that fails when RSS threshold flag is missing.

## Inputs / Outputs and Constraints

Inputs:
- benchmark-trend workflow compare matrix artifacts with leader `rssKb`.

Outputs:
- CI threshold steps now evaluate p99, coverage, and absolute RSS in one command invocation.

Constraints:
- RSS limits are conservative (`500000 KB`) in this slice to avoid over-tight initial gating.
- baseline-driven RSS drift controls remain optional and independent of this workflow-level absolute guard.

## Failure Modes and Diagnostics

- workflow contract tests fail deterministically if RSS threshold flags are removed from benchmark-trend workflow.
- runtime threshold failures report endpoint + measured RSS through `check_regression_thresholds.sh` diagnostics.

## Example Usage

CI step pattern now enforced:

```bash
benchmark-suite/scripts/check_regression_thresholds.sh \
  benchmark-suite/results/summaries/compare-matrix.json \
  --endpoint ping \
  --max-p99-ms 30 \
  --min-target-coverage 85 \
  --max-rss-kb 500000
```

## Tradeoffs and Next Steps

Tradeoffs:
- absolute RSS limits are intentionally generous initial guards rather than tuned endpoint-specific caps.

Next steps:
1. tighten RSS limits per endpoint once stable trend history is collected,
2. optionally add baseline RSS drift keys to live trend baseline files and enforce them in scheduled runs.
