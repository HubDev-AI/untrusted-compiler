# 245 M10 Slice: Step-Load Section in Markdown Reports

This chapter documents publishing step-load comparison signals in the benchmark markdown report.

## What it is

Updated:
- `benchmark-suite/scripts/publish_report.sh`
- `benchmark-suite/scripts/test_publish_report.sh`
- `benchmark-suite/scripts/testdata/sample-step-matrix.json`
- `benchmark-suite/README.md`

Key changes:
- `publish_report.sh` now accepts optional `step_matrix.json`,
- report includes a `Step-Load Signals` section when step matrix is provided,
- section includes:
  - endpoint/implementation counts,
  - total knee detections,
  - per-endpoint step leader summary.

## Why it exists

Step-load tooling is only useful if surfaced in final reporting. This slice exposes saturation/knee outcomes in the same artifact used for benchmark review.

## How it works internally

1. Validate optional `step_matrix.json` path if provided.
2. Include source pointer in report header metadata.
3. Render `## Step-Load Signals`:
   - aggregated counts from `step_matrix.summary`,
   - endpoint leader lines from `step_matrix.endpoints`.
4. If step matrix is omitted, render a clear “not provided” note.

## Inputs, outputs, and constraints

- Inputs:
  - optional 5th arg to `publish_report.sh`: `step_matrix.json`.
- Outputs:
  - markdown report with step-load section.
- Constraints:
  - step matrix schema must match aggregator output fields.

## Failure modes and diagnostics

- missing step matrix path when provided -> explicit file-not-found error.
- malformed step matrix structure -> jq render failure (same as other publish inputs).

## Example usage

```bash
benchmark-suite/scripts/publish_report.sh \
  benchmark-suite/results/summaries/compare-matrix.json \
  benchmark-suite/results/benchmark-report.md \
  baselines/sec-audit/default-secure-prod.hello.json \
  benchmark-suite/results/summaries/analysis.json \
  benchmark-suite/results/summaries/step-matrix.json
```

## Tradeoffs and next steps

- Tradeoff:
  - publish interface now has one more optional input artifact.
- Next:
  - auto-generate step matrix during orchestrated runs when step profiles are executed.
