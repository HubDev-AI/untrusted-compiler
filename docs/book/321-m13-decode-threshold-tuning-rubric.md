# 321 M13 Slice: Decode Threshold Tuning Rubric

This chapter defines deterministic rules for tuning the `decode` endpoint trend thresholds.

## What it is

A policy-level rubric for deciding when to change:
- workflow absolute `decode` thresholds in `.github/workflows/benchmark-trend.yml`,
- baseline regression policy in `benchmark-suite/baselines/node-decode-trend-baseline.json`.

## Why it exists

Without explicit rules, threshold changes become ad-hoc and hard to audit. M13 requires predictable trend governance with reproducible evidence.

## Current decode threshold posture (as of February 13, 2026)

Workflow guard (`benchmark-trend.yml`):
- `--max-p99-ms 80`
- `--min-target-coverage 60`

Baseline regression policy (`node-decode-trend-baseline.json`):
- `baselineP99Ms = 60.0`
- `baselineCoveragePct = 70.0`
- `maxP99RegressionPct = 25` (effective baseline p99 limit: `75.0ms`)
- `maxCoverageDropPct = 10` (effective baseline coverage limit: `60.0%`)

Decision:
- keep decode thresholds unchanged until live trend artifacts show sustained margin or sustained regression.

## Deterministic tuning rules

1. Tighten (stricter) thresholds only when all are true:
- at least 3 consecutive scheduled runs pass,
- median observed decode `p99` is at least 15% below current effective limit,
- median observed decode coverage is at least 10 points above minimum coverage limit.

2. Relax (looser) thresholds only when all are true:
- at least 2 consecutive scheduled runs fail the same decode check,
- rerun/verification excludes obvious infra noise (tool outage, missing dependency, partial run),
- change request includes explicit rationale in trend notes.

3. Change bounds per update:
- `max-p99-ms`: adjust by at most 10ms per change,
- `min-target-coverage`: adjust by at most 5 percentage points per change,
- baseline regression percentages: adjust by at most 5 points per change.

4. Atomic update requirement:
- update workflow threshold values and decode baseline file in the same commit,
- add a trend note entry documenting evidence and decision.

## Required evidence package for any threshold change

- compare-matrix artifact from each referenced run,
- threshold-check command output for decode,
- baseline file before/after diff,
- one short note in the M13 trend chapter stating decision and rationale.

## Tradeoffs and next steps

- Tradeoff:
  - stricter governance slows threshold changes but keeps trend policy auditable.
- Next:
  - populate first live trend note with observed decode metrics and apply this rubric for the first explicit keep/tune decision.
