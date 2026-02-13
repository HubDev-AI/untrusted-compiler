# 223 M10 Slice: Markdown Benchmark Report Generation

This chapter documents M10 report publishing from benchmark comparison artifacts.

## What it is

Added:
- `benchmark-suite/scripts/publish_report.sh`
- `benchmark-suite/scripts/test_publish_report.sh`
- Make target:
  - `make -C benchmark-suite publish-report`

The script generates a markdown benchmark report from:
- comparison matrix JSON,
- optional `sec.audit` JSON.

## Why it exists

M10 requires public-facing comparative reporting, not only machine artifacts. This slice creates a deterministic markdown report that can be committed or attached to release/benchmark notes.

## How it works internally

1. Validates matrix input and endpoint presence.
2. Writes report metadata (timestamp and source paths).
3. Renders endpoint leaders table from matrix `leader` rows.
4. Renders endpoint ranking lists from matrix `compared` rows.
5. Computes p99 spread per endpoint from recorded p99 values.
6. If provided, appends security posture summary from `sec.audit`:
   - policy name/hash,
   - finding count,
   - highest severity,
   - risk score,
   - top finding snippets.

## Inputs, outputs, and constraints

- Inputs:
  - `compare-matrix.json`
  - optional `sec.audit` JSON.
- Output:
  - markdown report (`results/benchmark-report.md` by default).
- Constraints:
  - matrix must contain at least one endpoint,
  - p99 parsing extracts numeric component from stored strings (e.g., `4.3ms`).

## Failure modes and diagnostics

- missing matrix file -> explicit error.
- matrix without endpoints -> explicit error.
- missing optional sec audit file when specified -> explicit error.
- malformed JSON -> `jq` parse failure.

## Example usage

```bash
make -C benchmark-suite compare-matrix
make -C benchmark-suite publish-report
```

Direct invocation:

```bash
benchmark-suite/scripts/publish_report.sh \
  benchmark-suite/results/summaries/compare-matrix.json \
  benchmark-suite/results/benchmark-report.md \
  baselines/sec-audit/default-secure-prod.hello.json
```

## Tradeoffs and next steps

- Tradeoff:
  - report ranking is throughput-first; it does not yet compute weighted tradeoff scores.
- Next:
  - include CPU/RSS/error-rate once report bundles carry those metrics,
  - support endpoint-specific narrative templates for publish-ready benchmark briefs.
