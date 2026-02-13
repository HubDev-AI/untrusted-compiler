# 226 M10 Slice: Analysis-Integrated Benchmark Report Publishing

This chapter documents wiring matrix analysis into benchmark report publishing and orchestration.

## What it is

Updated:
- `benchmark-suite/scripts/publish_report.sh`
- `benchmark-suite/scripts/test_publish_report.sh`
- `benchmark-suite/scripts/run_comparison_matrix.sh`
- `benchmark-suite/scripts/test_run_comparison_matrix.sh`
- `benchmark-suite/Makefile`

`publish_report.sh` now accepts optional matrix analysis input and renders a dedicated analysis section.

## Why it exists

M10 requires explicit tail-latency and failure-mode analysis in final benchmark outputs. This slice ensures analysis artifacts are generated and consumed by default in the report pipeline.

## How it works internally

1. `run_comparison_matrix.sh` now runs:
   - `compare_matrix.sh`
   - `analyze_matrix.sh`
   - `publish_report.sh` with analysis path.
2. `publish_report.sh` accepts:
   - `<matrix> <out> [sec_audit] [analysis]`
3. If analysis is provided, the report includes:
   - `## Matrix Analysis` summary,
   - endpoint-analysis finding snippets,
   - tail-latency section sourced from analysis metrics.
4. If analysis is omitted, tail-latency falls back to direct matrix-derived spread computation.

## Inputs, outputs, and constraints

- Inputs:
  - matrix (`compare-matrix.json`),
  - optional sec audit,
  - optional analysis (`analysis.json`).
- Outputs:
  - markdown benchmark report with analysis-aware sections.
- Constraints:
  - analysis and sec audit paths are validated when provided.

## Failure modes and diagnostics

- missing optional artifact path when provided -> explicit error.
- malformed analysis JSON -> `jq` parse failure.
- report generation remains deterministic given fixed inputs.

## Example usage

```bash
make -C benchmark-suite compare-matrix
make -C benchmark-suite analyze-matrix
make -C benchmark-suite publish-report
```

Or end-to-end:

```bash
make -C benchmark-suite bench-matrix
```

## Tradeoffs and next steps

- Tradeoff:
  - report currently shows top findings text only; it does not yet embed full per-endpoint analysis tables.
- Next:
  - include richer analysis tables and links to raw artifacts,
  - support policy-controlled pass/fail summary in markdown for CI publishing.
