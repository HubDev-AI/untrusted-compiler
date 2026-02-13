# 244 M10 Slice: Step-Analysis Comparison Matrix

This chapter documents cross-implementation aggregation for step-load analysis artifacts.

## What it is

Updated:
- `benchmark-suite/scripts/compare_step_matrix.sh`
- `benchmark-suite/scripts/test_compare_step_matrix.sh`
- `benchmark-suite/Makefile`
- `benchmark-suite/README.md`

Key changes:
- added `compare_step_matrix.sh` to combine `*-step-analysis.json` artifacts,
- supports optional implementation and endpoint scoping,
- emits per-endpoint rankings using knee headroom (`kneeAtTargetRps`) and achieved ratio,
- added Make target `compare-step-matrix`.

## Why it exists

Step analysis was previously per-implementation only. Comparing saturation behavior across implementations required manual file-by-file inspection. This slice provides deterministic multi-implementation step comparison output.

## How it works internally

1. Load selected step-analysis artifacts (scoped or full directory scan).
2. Normalize rows:
   - knee detection flags,
   - knee target/observed RPS,
   - achieved-ratio and p99 bounds.
3. Group by endpoint and sort compared rows by:
   - `kneeAtTargetRps` descending,
   - `achievedRatioMin` descending.
4. Emit endpoint leaders and summary counters.

## Inputs, outputs, and constraints

- Inputs:
  - `compare_step_matrix.sh <summaries_dir> <out.json> [impls_csv] [endpoints_csv]`
- Outputs:
  - step comparison matrix JSON (`version: 0.1`).
- Constraints:
  - when scoped lists are provided, every selected impl/endpoint analysis file must exist.

## Failure modes and diagnostics

- missing selected analysis artifact -> explicit path error.
- empty directory / no valid rows -> explicit failure.

## Example usage

```bash
make -C benchmark-suite compare-step-matrix IMPLS=sec4,node,go,rust ENDPOINTS=decode
```

## Tradeoffs and next steps

- Tradeoff:
  - ranking currently uses simple deterministic ordering; no statistical confidence model is applied.
- Next:
  - add step-matrix section to markdown report publishing and include per-endpoint knee leaders.
