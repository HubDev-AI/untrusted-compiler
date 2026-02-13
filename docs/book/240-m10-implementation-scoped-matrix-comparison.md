# 240 M10 Slice: Implementation-Scoped Matrix Comparison

This chapter documents implementation scoping for matrix comparison to prevent stale non-selected reports from affecting filtered runs.

## What it is

Updated:
- `benchmark-suite/scripts/compare_matrix.sh`
- `benchmark-suite/scripts/run_comparison_matrix.sh`
- `benchmark-suite/scripts/test_compare_matrix.sh`
- `benchmark-suite/scripts/test_run_comparison_matrix.sh`
- `benchmark-suite/Makefile`
- `benchmark-suite/README.md`

Key changes:
- `compare_matrix.sh` now accepts optional `impls_csv`,
- when implementation list is provided, comparison uses only matching `<impl>-report.json` files and requires each to exist,
- orchestrator now passes selected implementation set into compare step,
- Make target `compare-matrix` now scopes comparison by `IMPLS`.

## Why it exists

Even with endpoint filtering fixed, compare-matrix still scanned all report files under summaries. That allowed stale reports from non-selected implementations to leak into current run analysis.

## How it works internally

1. Orchestrator runs selected implementations.
2. Per-implementation reports are generated with current run scope.
3. Compare step receives `impls_csv` and loads only those report files.
4. Missing selected report file fails immediately.
5. Analyze/publish operate on scoped matrix output.

## Inputs, outputs, and constraints

- Inputs:
  - optional `impls_csv` in `compare_matrix.sh`,
  - `IMPLS` in Make/orchestrator.
- Outputs:
  - matrix JSON containing only selected implementation rows.
- Constraints:
  - selected implementations must have report bundles present.

## Failure modes and diagnostics

- missing report for selected impl:
  - `missing report file for impl=<impl>: <path>`
- empty/invalid implementation list still handled by orchestrator validation before compare step.

## Example usage

```bash
benchmark-suite/scripts/compare_matrix.sh benchmark-suite/results/summaries benchmark-suite/results/summaries/compare-matrix.json "sec4,node"
```

## Tradeoffs and next steps

- Tradeoff:
  - compare step becomes stricter and fails if selected impl reports are absent.
- Next:
  - expose implementation scope in markdown publish header alongside endpoint scope.
