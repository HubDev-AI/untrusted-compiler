# 1010 M39 Slice: Benchmark RSS Regression Threshold Guard

## What It Is

This slice extends `benchmark-suite/scripts/check_regression_thresholds.sh` with memory guard support:

- absolute RSS guard: `--max-rss-kb <value>`,
- baseline RSS drift guard: `baselineRssKb` + `maxRssRegressionPct`.

It keeps existing p99 and target-coverage checks unchanged.

## Why It Exists

After adding RSS visibility into benchmark artifacts (`1009`), load-hardening needed a deterministic way to fail regression checks on memory growth, not only latency/throughput drift.

## How It Works Internally

1. Leader RSS extraction:
   - reads endpoint leader `rssKb` from `compare-matrix.json`.
   - validates numeric shape when present.

2. Absolute threshold mode:
   - when `--max-rss-kb` is provided, command fails if leader `rssKb` exceeds that value.
   - if leader RSS is missing while RSS guard is requested, command fails deterministically.

3. Baseline threshold mode:
   - baseline files may now include:
     - `baselineRssKb`
     - `maxRssRegressionPct` (default `20` when absent).
   - if `baselineRssKb` exists, command computes limit:
     - `baselineRssKb * (1 + maxRssRegressionPct / 100)`,
   - and fails on RSS regressions above limit.

4. Output contract:
   - success summary now includes leader RSS when available (`rssKb=<value>`),
   - p99/coverage baseline checks remain unchanged.

## Inputs / Outputs and Constraints

Inputs:
- `compare-matrix.json` endpoint leader rows with `rssKb`,
- optional CLI guard (`--max-rss-kb`),
- optional baseline file fields (`baselineRssKb`, `maxRssRegressionPct`).

Outputs:
- deterministic PASS/FAIL exit status with explicit threshold diagnostics.

Constraints:
- RSS checks only execute when explicitly requested (CLI flag) or when baseline RSS keys are present.
- Existing baseline files without RSS fields remain compatible.

## Failure Modes and Diagnostics

- invalid `--max-rss-kb` values are rejected with usage-level errors,
- missing/non-numeric leader RSS under active RSS guard is rejected deterministically,
- RSS threshold and baseline regression failures report endpoint + measured RSS + computed limits.

## Example Usage

Absolute guard:

```bash
benchmark-suite/scripts/check_regression_thresholds.sh \
  benchmark-suite/results/summaries/compare-matrix.json \
  --endpoint ping \
  --max-p99-ms 30 \
  --min-target-coverage 85 \
  --max-rss-kb 120000
```

Baseline guard:

```bash
benchmark-suite/scripts/check_regression_thresholds.sh \
  benchmark-suite/results/summaries/compare-matrix.json \
  --endpoint ping \
  --baseline benchmark-suite/scripts/testdata/sample-trend-baseline-ping.json
```

## Tradeoffs and Next Steps

Tradeoffs:
- current guard checks single leader RSS values from matrix artifacts, not time-series/peak memory envelopes.

Next steps:
1. tune implementation-specific RSS thresholds in live baselines once stable hardware-normalized evidence is captured,
2. consider optional peak-RSS artifact integration for long-duration and step-load phases.
