# M13 Slice: Closure Evidence Refresh Command

This slice adds a single operator command that refreshes both remaining strict-closure evidence tracks and immediately verifies closure status.

## Why this exists

Two closure gates remain operationally coupled:

1. `M10-A`: cross-implementation compare matrix must include `sec4/go/node/rust`.
2. `M13-A`: trend chapter must contain at least one live `Trend Entry` block.

Before this slice, each gate had its own updater command. Operators had to run them manually in sequence, then run closure audit separately.

## What was implemented

1. New command:
- `scripts/refresh-closure-evidence-from-ci.sh`

2. Command behavior:
- Step 1: runs M10 updater:
  - `benchmark-suite/scripts/update_cross_impl_matrix_from_ci.sh`
- Step 2: runs M13 updater:
  - `benchmark-suite/scripts/update_trend_note_from_ci.sh`
- Step 3: runs benchmark evidence quality check:
  - `scripts/check-benchmark-evidence-quality.sh`
- Step 4: runs strict closure audit:
  - `scripts/check-milestone-closure.sh --fail-on-pending`

3. Supported modes:
- CI artifact mode (default): fetches latest successful artifacts.
- Local deterministic mode:
  - `--matrix <path>` for M10 matrix source
  - `--entry <path>` for M13 trend-entry source
- Strict evidence-quality mode is default:
  - internally runs `check-benchmark-evidence-quality.sh --fail-on-warning`
  - propagates strict/default quality mode into M10 importer (`update_cross_impl_matrix_from_ci.sh`)
  - use `--quality-allow-warning` only for local fallback/debug scenarios
- `--dry-run` for command-plan preview.

4. Test coverage:
- `scripts/test-refresh-closure-evidence-from-ci.sh`
- Validates:
  - dry-run command composition,
  - strict-vs-allow quality mode command composition and importer propagation,
  - local fixture-backed matrix and trend-note import,
  - strict closure check passes for the provided fixture targets.

5. Smoke CI coverage:
- `.github/workflows/benchmark-smoke.yml` now runs:
  - `scripts/test-refresh-closure-evidence-from-ci.sh`

## Operator usage

Default live mode:

```bash
scripts/refresh-closure-evidence-from-ci.sh
```

Deterministic local mode:

```bash
scripts/refresh-closure-evidence-from-ci.sh \
  --matrix benchmark-suite/scripts/testdata/sample-cross-impl-compare-matrix.json \
  --entry /tmp/trend-note-entry.md \
  --target-matrix benchmark-suite/results/summaries/compare-matrix.json \
  --trend-note docs/book/322-m13-first-trend-run-results-note.md
```

Dry run:

```bash
scripts/refresh-closure-evidence-from-ci.sh --dry-run
```

## Notes

This command reduces operator error and makes closure refresh reproducible. Live completion still depends on CI artifact availability and GitHub authentication.
