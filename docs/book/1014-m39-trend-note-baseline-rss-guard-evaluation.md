# 1014 M39 Slice: Trend Note Baseline RSS Guard Evaluation

## What It Is

This slice extends trend-note baseline guard evaluation to include optional memory regression checks:

- when baseline files include `baselineRssKb`, baseline pass/fail now also evaluates leader RSS drift,
- existing p99/coverage baseline checks remain unchanged.

## Why It Exists

Trend notes already displayed leader RSS (`1011`), and threshold tooling already supported RSS guards (`1010`), but baseline verdicts in rendered trend notes still ignored memory drift.

This slice aligns trend-note baseline status with full throughput/latency/memory posture.

## How It Works Internally

1. Renderer input extraction:
   - `render_trend_note_entry.sh` now extracts leader `rssKb` as numeric input (or empty when missing).

2. Baseline evaluation:
   - baseline files continue requiring `baselineP99Ms` + `baselineCoveragePct` for baseline mode,
   - when optional `baselineRssKb` is present:
     - renderer computes RSS limit using `maxRssRegressionPct` (default `20`),
     - baseline status is `fail` if leader RSS is missing/invalid or exceeds limit.

3. Output behavior:
   - table shape is unchanged from `1011` (`RSS (KB)` column already present),
   - baseline status now reflects RSS-aware failure when applicable.

4. Test coverage:
   - added focused renderer test fixture with baseline RSS keys and over-limit leader RSS,
   - asserts row-level `Baseline Guard = fail` while absolute guard remains `pass`.

## Inputs / Outputs and Constraints

Inputs:
- compare matrix leader rows with optional `rssKb`,
- baseline files under `--baseline-dir` with optional `baselineRssKb` and `maxRssRegressionPct`.

Outputs:
- trend-note baseline guard status that may fail on memory drift.

Constraints:
- RSS baseline checks are opt-in via baseline file keys.
- if `baselineRssKb` is absent, baseline behavior remains p99/coverage-only.

## Failure Modes and Diagnostics

- malformed or missing core baseline metrics still mark baseline status as `invalid`,
- RSS baseline enabled + missing/invalid leader RSS marks baseline status as `fail`.

## Example Usage

```bash
benchmark-suite/scripts/render_trend_note_entry.sh \
  benchmark-suite/results/summaries/compare-matrix.json \
  --endpoints ping,decode \
  --baseline-dir benchmark-suite/baselines
```

If `node-ping-trend-baseline.json` contains `baselineRssKb`, baseline status for `ping` now includes RSS drift in pass/fail.

## Tradeoffs and Next Steps

Tradeoffs:
- baseline verdict is now stricter when RSS baselines are supplied, which can increase fail frequency until baselines are tuned.

Next steps:
1. add RSS baseline keys to production trend baseline files once stable live values are captured,
2. optionally expose baseline-limit details in rendered markdown for faster operator diagnosis.
