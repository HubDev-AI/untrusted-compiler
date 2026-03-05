# 1547 M39 Slice: Workbench benchmark compact HTML report

## What it is

This slice adds visual benchmark report outputs for workbench matrix/full runs:

- new renderer script:
  - `benchmark-suite/scripts/render_workbench_benchmark_report_html.sh`
- workbench matrix runner now emits HTML alongside existing markdown:
  - `benchmark-suite/scripts/run_workbench_benchmark_matrix.sh`
  - output: `benchmark-suite/results/workbench-benchmark-report.html`
- workbench full-suite runner now carries dedicated HTML output path:
  - `benchmark-suite/scripts/run_workbench_full_benchmark_suite.sh`
  - output: `benchmark-suite/results/workbench-full-benchmark-report.html`

## Why it exists

Operators needed a directly viewable report with all backend results in one place, without horizontal scrolling and without re-running benchmarks just to inspect data.

## How it works internally

1. The renderer reads existing JSON artifacts:
   - runs summary,
   - compare matrix,
   - analysis output.
2. It builds one self-contained HTML file with:
   - run metadata and severity summary,
   - leader-win counts per implementation,
   - per-endpoint ranked rows for all implementations,
   - endpoint findings from analysis.
3. The matrix runner now invokes the renderer automatically after compare/analysis/markdown generation.
4. The full-suite runner now forwards `--out-report-html` into the matrix lane and records `reportHtml` in full-suite run manifests.
5. A new make target can re-render HTML from existing JSON artifacts:
   - `make -C benchmark-suite workbench-bench-report-html`

## Inputs, outputs, and constraints

- Inputs:
  - `results/summaries/workbench-benchmark-runs.json`
  - `results/summaries/workbench-benchmark-compare-matrix.json`
  - `results/summaries/workbench-benchmark-analysis.json`
- Outputs:
  - `results/workbench-benchmark-report.html`
  - `results/workbench-full-benchmark-report.html`
- Constraint:
  - no additional benchmark execution is required to regenerate HTML when JSON artifacts already exist.

## Failure modes and diagnostics

- Missing input file: renderer exits with usage error and prints missing path.
- Invalid JSON payload: `jq` fails and renderer exits non-zero.
- Runner dry-run mode now prints the HTML render command in the execution plan.

## Tradeoffs and next steps

- Tradeoff: report is static HTML with embedded JSON and simple client-side rendering (no external assets, no framework).
- Benefit: deterministic artifact, portable viewing, and compact layout for quick cross-backend review.
- Next step: optionally add run-to-run delta visuals using repeated-suite artifacts.
