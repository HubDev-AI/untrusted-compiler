# 460 M13 Slice: Trend Note Update Local Fallback Mode

This chapter documents making the trend-note updater resilient when remote CI artifact fetch is unavailable.

## 1) What it is

`benchmark-suite/scripts/update_trend_note_from_ci.sh` now supports:

- `--prefer-local`: render trend entry directly from local `compare-matrix.json`
- automatic fallback to local render when remote fetch fails
- `--local-matrix <path>` override for deterministic local inputs

## 2) Why it exists

In some environments, remote workflow/artifact fetch can fail (missing workflow on default branch, missing `gh` auth, restricted network). The update flow should still work from local benchmark evidence.

## 3) How it works internally

1. If `--entry` is provided, import directly.
2. If `--prefer-local` is set, render entry from local matrix and import.
3. Otherwise try remote fetch first.
4. If fetch fails or no downloaded `trend-note-entry.md` is found, render from local matrix and import.
5. Temporary rendered entries are cleaned up automatically.

## 4) Inputs, outputs, constraints

Inputs:

- remote artifact (optional)
- local compare matrix (`benchmark-suite/results/summaries/compare-matrix.json` by default)

Outputs:

- imported `## Trend Entry (...)` block in target chapter

Constraints:

- local fallback requires a valid compare matrix file.
- renderer schema expectations must be satisfied by matrix rows.

## 5) Failure modes and diagnostics

- missing local fallback matrix:
  - `local compare-matrix fallback is missing: <path>`
- remote fetch failure:
  - warning is emitted, updater falls back to local render.

## 6) Example usage

```bash
benchmark-suite/scripts/update_trend_note_from_ci.sh --prefer-local
benchmark-suite/scripts/update_trend_note_from_ci.sh --local-matrix benchmark-suite/results/summaries/compare-matrix.json
```

## 7) Trade-offs and next steps

Trade-offs:

- fallback mode may use local non-constant-rate evidence if that is what exists in `compare-matrix.json`.

Next steps:

- attach explicit run-mode note in updater output when local fallback is used to avoid misinterpreting threshold posture.

## Verification

- `benchmark-suite/scripts/test_update_trend_note_from_ci.sh`
- `benchmark-suite/scripts/test_render_trend_note_entry.sh`
- `benchmark-suite/scripts/test_import_trend_note_entry.sh`
