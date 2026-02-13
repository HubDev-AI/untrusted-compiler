# 222 M10 Slice: Multi-Endpoint Comparison Matrix

This chapter documents M10 matrix reporting across all benchmarked endpoints in implementation report bundles.

## What it is

Added:
- `benchmark-suite/scripts/compare_matrix.sh`
- `benchmark-suite/scripts/test_compare_matrix.sh`
- Make target:
  - `make -C benchmark-suite compare-matrix`

The script emits one comparison artifact containing ranked rows for every endpoint present in `*-report.json` files.

## Why it exists

Single-endpoint comparison output is useful for quick checks, but M10 requires reproducible cross-language analysis across the full endpoint set (`ping`, `decode`, `users-post`). This slice removes manual per-endpoint aggregation.

## How it works internally

1. Reads all `*-report.json` files in a report directory.
2. Flattens each report’s `summaries[]` into normalized rows:
   - `impl`
   - `endpoint`
   - `targetRps`
   - `requestsPerSec`
   - `p99`
3. Groups rows by endpoint.
4. Sorts each endpoint group by `requestsPerSec` descending.
5. Emits endpoint entries with:
   - `compared` rows
   - `leader` row

## Inputs, outputs, and constraints

- Inputs:
  - implementation report bundles from `build_report.sh`.
- Output:
  - comparison matrix JSON (`results/summaries/compare-matrix.json` by default).
- Constraints:
  - endpoints absent from all reports are not emitted.
  - malformed report JSON fails fast via `jq`.

## Failure modes and diagnostics

- no report files -> explicit error.
- no usable summaries in report files -> explicit error.
- invalid/malformed JSON -> `jq` parse failure.

## Example usage

```bash
make -C benchmark-suite compare-matrix
```

Direct invocation:

```bash
benchmark-suite/scripts/compare_matrix.sh \
  benchmark-suite/results/summaries \
  benchmark-suite/results/summaries/compare-matrix.json
```

## Tradeoffs and next steps

- Tradeoff:
  - ranking currently uses throughput only (`requestsPerSec`) and includes `p99` as a column, not as a weighted score.
- Next:
  - add optional composite ranking profiles (latency-first vs throughput-first),
  - include error-rate/CPU/RSS once those fields are standardized in report bundles.
