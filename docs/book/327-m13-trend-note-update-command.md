# 327 M13 Slice: Trend Note Update Command

This chapter documents the one-command helper that fetches and imports trend-note entries into chapter 322.

## What it is

Updated:
- `benchmark-suite/scripts/update_trend_note_from_ci.sh`
- `benchmark-suite/scripts/test_update_trend_note_from_ci.sh`
- `.github/workflows/benchmark-smoke.yml`
- `docs/book/322-m13-first-trend-run-results-note.md`
- `docs/05-sec4-master-roadmap.md`

## Why it exists

Fetch and import helpers existed separately, but operators still had to manually chain them. This command provides a single deterministic path for updating trend notes from CI artifacts.

## How it works internally

`update_trend_note_from_ci.sh`:
1. fetches latest successful trend artifact package (unless `--entry` is provided),
2. locates `trend-note-entry.md`,
3. imports it into chapter `322` via `--replace-existing` mode, so same-day reruns refresh existing entry content.

`--dry-run` prints the composed command sequence.

## Tests

`test_update_trend_note_from_ci.sh` validates:
- dry-run command composition,
- local import path via `--entry`,
- chapter update outcome.

Benchmark smoke CI now runs this test.

## Inputs, outputs, and constraints

- Inputs:
  - repo slug/output directory (for fetch path) or explicit `--entry`,
  - chapter path (defaults to chapter `322`).
- Output:
  - updated chapter with imported or refreshed trend entry.
- Constraints:
  - live fetch mode still requires valid `gh` auth and network.

## Example usage

```bash
benchmark-suite/scripts/update_trend_note_from_ci.sh \
  --repo HubDev-AI/untrusted-compiler
```

## Tradeoffs and next steps

- Tradeoff:
  - helper currently assumes default artifact/workflow naming conventions.
- Next:
  - optionally externalize workflow/artifact defaults into a shared config file.
