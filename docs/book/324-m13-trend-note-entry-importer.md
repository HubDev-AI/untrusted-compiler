# 324 M13 Slice: Trend Note Entry Importer

This chapter documents the importer that appends CI-rendered trend entries into the M13 trend-note chapter.

## What it is

Updated:
- `benchmark-suite/scripts/import_trend_note_entry.sh`
- `benchmark-suite/scripts/test_import_trend_note_entry.sh`
- `.github/workflows/benchmark-smoke.yml`
- `docs/book/322-m13-first-trend-run-results-note.md`
- `docs/05-sec4-master-roadmap.md`

## Why it exists

Renderer output gives a deterministic note fragment, but chapter updates were still manual. The importer provides a single idempotent command to append entries and avoid duplicate insertion.

## How it works internally

`import_trend_note_entry.sh`:
1. validates input entry + chapter paths,
2. requires a heading of form `## Trend Entry (<date>)`,
3. checks whether that heading already exists in the chapter,
4. appends entry only when missing.

Deduplication key is the entry heading line.

## Tests

`test_import_trend_note_entry.sh` verifies:
- first import appends the entry,
- second import with same entry is idempotent (no duplicate heading).

Benchmark smoke CI now executes this test.

## Inputs, outputs, and constraints

- Inputs:
  - rendered entry markdown file (for example `trend-note-entry.md` from benchmark artifacts),
  - target chapter path (defaults to `docs/book/322-m13-first-trend-run-results-note.md`).
- Output:
  - updated trend-note chapter with appended entry when not already present.
- Constraints:
  - entry heading must match `## Trend Entry (<date>)` format for deduplication.

## Example usage

```bash
benchmark-suite/scripts/import_trend_note_entry.sh \
  --entry benchmark-suite/results/summaries/trend-note-entry.md
```

## Tradeoffs and next steps

- Tradeoff:
  - heading-based deduplication assumes unique date headings.
- Next:
  - optionally support stricter dedupe keys (heading + source hash) if same-day multiple entries become common.
