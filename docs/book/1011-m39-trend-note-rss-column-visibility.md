# 1011 M39 Slice: Trend Note RSS Column Visibility

## What It Is

This slice extends trend-note rendering/import fixtures to include leader memory signal:

- trend-note table now includes `RSS (KB)` column sourced from leader `rssKb`,
- import/update tests are aligned with the new column shape.

## Why It Exists

After benchmark matrix/report artifacts began carrying RSS and threshold guards added memory checks, trend governance notes still surfaced only p99 and coverage.

This slice keeps operator-facing trend notes aligned with throughput/latency/memory visibility.

## How It Works Internally

1. `render_trend_note_entry.sh` now:
   - extracts leader `rssKb` from matrix rows,
   - renders `RSS (KB)` as numeric text when present or `n/a` when absent,
   - keeps existing absolute/baseline guard evaluation behavior unchanged.

2. Row shape updates:
   - missing leader rows now emit an extra placeholder column for RSS,
   - normal rows append RSS before guard-status columns.

3. Tests:
   - `test_render_trend_note_entry.sh` assertions updated for the new column,
   - `test_import_trend_note_entry.sh` fixtures/header expectations updated to preserve idempotent replace/import behavior with the expanded table.

## Inputs / Outputs and Constraints

Inputs:
- trend compare-matrix artifact rows with optional `rssKb`.

Outputs:
- markdown trend-note entries with `RSS (KB)` column.

Constraints:
- when leader `rssKb` is missing/null, renderer outputs `n/a` (no forced failure in this slice).

## Failure Modes and Diagnostics

- missing endpoint leaders still render explicit `missing` row markers,
- import script behavior remains deterministic: duplicate heading skip and `--replace-existing` rewrite semantics are unchanged.

## Example Usage

```bash
benchmark-suite/scripts/render_trend_note_entry.sh \
  benchmark-suite/scripts/testdata/sample-trend-compare-matrix.json \
  --date 2026-02-19
```

## Tradeoffs and Next Steps

Tradeoffs:
- trend-note RSS is display-oriented in this slice; guard scoring still follows existing absolute/baseline status logic.

Next steps:
1. optionally add explicit RSS guard-status columns if operator workflows need separate memory pass/fail surfacing,
2. propagate baseline RSS details into trend-note footnotes when baseline files are RSS-enabled.
