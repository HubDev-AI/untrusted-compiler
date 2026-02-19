# 1013 M39 Slice: Benchmark Evidence Quality RSS Warning Enforcement

## What It Is

This slice extends benchmark evidence-quality gating to include leader memory signal quality:

- `scripts/check-benchmark-evidence-quality.sh` now emits `WARN` when leader `rssKb` is missing/invalid.
- compare-row identity checks now include `rssKb` parity between `leader` and `compared[]` rows.

## Why It Exists

After adding RSS metrics, thresholds, trend rendering, and CI threshold wiring, quality posture checks still allowed missing memory signal to pass silently.

This slice ensures memory evidence completeness is enforced at the same governance layer as latency/run-mode quality checks.

## How It Works Internally

1. Quality checklist update:
   - adds `leader rssKb > 0` PASS check.
   - emits warning diagnostic `leader rssKb missing/invalid` when absent or non-numeric.

2. Leader membership parity:
   - normalization now carries `rssKb`,
   - leader-vs-compared row equality includes `rssKb` to prevent stale leader identity matches when memory fields drift.

3. Test fixture refresh:
   - PASS fixture (`sample-cross-impl-compare-matrix.json`) now includes concrete `rssKb` values.
   - warn-path tests assert RSS warning behavior and `--fail-on-warning` compatibility.

## Inputs / Outputs and Constraints

Inputs:
- compare matrix endpoint entries with leader/compared rows.

Outputs:
- quality report now includes RSS-specific PASS/WARN lines.

Constraints:
- RSS quality remains a warning class (not structural FAIL) unless `--fail-on-warning` is enabled.

## Failure Modes and Diagnostics

- malformed matrix structure still returns FAIL (`exit 2`) as before.
- missing/invalid RSS now increases warning count and can fail strict quality mode (`--fail-on-warning`).

## Example Usage

```bash
scripts/check-benchmark-evidence-quality.sh \
  --matrix benchmark-suite/results/summaries/compare-matrix.json \
  --fail-on-warning
```

## Tradeoffs and Next Steps

Tradeoffs:
- RSS enforcement at this layer is presence/validity-oriented; absolute/baseline threshold strictness stays in `check_regression_thresholds.sh`.

Next steps:
1. add optional per-endpoint minimum RSS-data coverage checks for multi-endpoint matrices,
2. align quality summaries with any future peak-memory artifact fields when introduced.
