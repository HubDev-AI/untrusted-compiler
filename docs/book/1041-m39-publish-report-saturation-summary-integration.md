# 1041 M39 Slice: Publish Report Saturation Summary Integration

## What It Is

This slice integrates LASM saturation tuning summary artifacts into benchmark report publishing.

Updated files:

- `benchmark-suite/scripts/publish_report.sh`
- `benchmark-suite/scripts/test_publish_report.sh`
- `benchmark-suite/scripts/testdata/sample-saturation-boost-summary.md`
- `benchmark-suite/Makefile`

## Why It Exists

Saturation tuning output was available as a dedicated markdown summary, but the main benchmark report did not surface it.

That separation made final report review harder during handoff. This slice brings recommended boost-step context into the primary report artifact.

## How It Works Internally

1. New optional input:
   - `publish_report.sh` now accepts:
     - `[saturation_summary.md]` as optional 6th argument.

2. Validation when provided:
   - summary file must exist,
   - summary must contain required lines:
     - `- Recommended boost step: ...`
     - `- Selection mode: ...`

3. Report output:
   - adds section:
     - `## LASM Saturation Boost Tuning`
   - section includes:
     - summary source path,
     - recommended boost step,
     - selection mode,
     - pass runs,
     - optional verification requests/sec when available.

4. Makefile integration:
   - `publish-report` now accepts optional `SATURATION_SUMMARY=<path>`.
   - default behavior is unchanged when `SATURATION_SUMMARY` is unset.

## Inputs / Outputs and Constraints

Inputs:
- existing compare matrix + sec audit + optional analysis/step matrix,
- optional saturation summary markdown artifact.

Outputs:
- benchmark report markdown now optionally includes saturation tuning summary section.

Constraints:
- saturation summary integration is strict about required key lines to catch malformed/mismatched summary artifacts early.

## Failure Modes and Diagnostics

- missing saturation summary file:
  - `saturation summary file not found: <path>`
- malformed summary shape:
  - `saturation summary missing recommended boost step line: <path>`
  - `saturation summary missing selection mode line: <path>`

## Example Usage

Direct script usage:

```bash
benchmark-suite/scripts/publish_report.sh \
  benchmark-suite/results/summaries/compare-matrix.json \
  benchmark-suite/results/benchmark-report.md \
  baselines/sec-audit/default-secure-prod.hello.json \
  benchmark-suite/results/summaries/analysis.json \
  "" \
  benchmark-suite/results/summaries/sec4-lasm-cluster-saturation-boost-summary.md
```

Make target:

```bash
make -C benchmark-suite publish-report \
  SATURATION_SUMMARY=results/summaries/sec4-lasm-cluster-saturation-boost-summary.md
```

## Tradeoffs and Next Steps

Tradeoffs:
- report generation now enforces extra summary-shape checks when saturation summary is supplied.
- this improves integration quality but can fail fast on malformed ad-hoc summary files.

Next steps:
1. wire saturation summary artifact generation and report publish into a single workflow target.
2. continue runtime/proxy throughput optimization while tracking recommendation drift in published reports.
