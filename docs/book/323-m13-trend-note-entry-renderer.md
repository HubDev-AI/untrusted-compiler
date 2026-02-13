# 323 M13 Slice: Trend Note Entry Renderer

This chapter documents the deterministic markdown renderer for benchmark trend-note entries.

## What it is

Updated:
- `benchmark-suite/scripts/render_trend_note_entry.sh`
- `benchmark-suite/scripts/test_render_trend_note_entry.sh`
- `benchmark-suite/scripts/testdata/sample-trend-compare-matrix.json`
- `.github/workflows/benchmark-smoke.yml`
- `docs/book/322-m13-first-trend-run-results-note.md`
- `docs/05-sec4-master-roadmap.md`

## Why it exists

Trend-note updates were manual and prone to formatting drift. M13-S2 needs repeatable, artifact-driven note entries so operators can append live observations consistently.

## How it works internally

`render_trend_note_entry.sh` reads `compare-matrix.json` leader metrics for selected endpoints and renders markdown:
- endpoint leader impl,
- numeric `p99` and coverage values,
- absolute-guard pass/fail (`ping` and `decode` defaults),
- baseline-guard pass/fail when baseline files exist.

It outputs either:
- stdout (default), or
- a file via `--out`.

## Tests

`test_render_trend_note_entry.sh` validates:
- heading/date output,
- per-endpoint rows (pass for `ping`, fail for `decode` fixture),
- overall absolute/baseline status summaries.

`benchmark-smoke.yml` now runs this test in CI.

## Inputs, outputs, and constraints

- Inputs:
  - `compare-matrix.json`,
  - optional baseline directory (`benchmark-suite/baselines` by default).
- Output:
  - markdown trend-note entry block.
- Constraints:
  - absolute guard defaults are currently endpoint-specific for `ping` and `decode`.

## Example usage

```bash
benchmark-suite/scripts/render_trend_note_entry.sh \
  benchmark-suite/results/summaries/compare-matrix.json \
  --endpoints ping,decode \
  --date 2026-02-13 \
  --baseline-dir benchmark-suite/baselines
```

## Tradeoffs and next steps

- Tradeoff:
  - endpoint absolute-guard values are currently script defaults, not auto-read from workflow config.
- Next:
  - optionally source guard values from a shared threshold config file to avoid duplicated constants.
