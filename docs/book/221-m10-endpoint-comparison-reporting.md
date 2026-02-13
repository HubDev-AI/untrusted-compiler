# 221 M10 Slice: Endpoint Comparison Reporting

This chapter documents M10 comparison reporting support over per-implementation benchmark bundles.

## What it is

Added:
- `benchmark-suite/scripts/compare_reports.sh`
- `benchmark-suite/scripts/test_compare_reports.sh`
- test fixtures for sample impl report bundles
- Make target:
  - `make -C benchmark-suite compare`

The script builds an endpoint-level comparison artifact from `*-report.json` files.

## Why it exists

M10 requires cross-language comparison outputs, not just isolated per-impl runs. This slice produces a deterministic comparison artifact with quality-aware ranking and p99 columns per endpoint.

## How it works internally

1. Reads report files matching `*-report.json` from a report directory.
2. Extracts the requested endpoint summary from each report.
3. Builds comparison rows:
   - `impl`
   - `targetRps`
   - `requestsPerSec`
   - `p99`
   - `loadGenerator` (defaults to `wrk2` if missing)
   - `constantRate` (defaults to `true` if missing)
4. Sorts rows with constant-rate quality priority:
   - `constantRate=true` rows first,
   - then `requestsPerSec` descending.
5. Emits:
   - `compared` array,
   - `leader` row.

## Inputs, outputs, and constraints

- Inputs:
  - report bundles from `build_report.sh`.
- Outputs:
  - comparison JSON (default: `results/summaries/compare-ping.json`).
- Constraints:
  - only report files containing the requested endpoint are included.
  - current Make target compares `ping`; other endpoints use direct script invocation.

## Failure modes and diagnostics

- no report files -> explicit error.
- report files without requested endpoint -> explicit error.
- malformed report JSON -> jq parse failure.

## Example usage

```bash
make -C benchmark-suite compare
```

Direct endpoint compare:

```bash
benchmark-suite/scripts/compare_reports.sh \
  benchmark-suite/results/summaries \
  decode \
  benchmark-suite/results/summaries/compare-decode.json
```

## Tradeoffs and next steps

- Tradeoff:
  - ranking currently prioritizes constant-rate run quality and then throughput, while `p99` remains a reported (not weighted) column.
- Next:
  - extend comparison schema with error rate, CPU, and RSS once those metrics are captured per report,
  - generate combined multi-endpoint comparative report for publication.
