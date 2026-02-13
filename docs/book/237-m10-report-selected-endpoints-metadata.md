# 237 M10 Slice: Report Selected-Endpoints Metadata

This chapter documents adding endpoint-selection metadata to per-implementation benchmark reports.

## What it is

Updated:
- `benchmark-suite/scripts/build_report.sh`
- `benchmark-suite/scripts/test_build_report.sh`
- `benchmark-suite/README.md`

Key changes:
- report bundle now includes `selectedEndpoints` field,
- when orchestrator/report bundling is endpoint-filtered, selected endpoint list is embedded in output report JSON,
- metadata is `null` when no explicit endpoint filter was provided.

## Why it exists

Filtered matrix runs and full runs can both produce valid reports. Downstream consumers (comparison scripts, CI, human review) need a reliable indicator of scope to avoid misinterpreting partial results as full-suite outputs.

## How it works internally

1. `build_report.sh` accepts optional `endpoints_csv`.
2. Endpoint list is normalized and serialized to JSON array.
3. Report output includes:
   - `selectedEndpoints: ["ping", ...]` for filtered runs,
   - `selectedEndpoints: null` for unfiltered/default behavior.

## Inputs, outputs, and constraints

- Inputs:
  - optional `endpoints_csv` argument to `build_report.sh`.
- Outputs:
  - report JSON with explicit endpoint-scope metadata.
- Constraints:
  - metadata reflects requested bundle scope, not implicit detection.

## Failure modes and diagnostics

- malformed endpoint strings are normalized by existing orchestrator/report filtering flow.
- missing selected endpoint summaries still fail before report output (see previous endpoint-scoped bundling slice).

## Example output fragment

```json
{
  "impl": "ailang",
  "selectedEndpoints": ["ping", "decode"]
}
```

## Tradeoffs and next steps

- Tradeoff:
  - one additional metadata field in report schema.
- Next:
  - surface `selectedEndpoints` in markdown publish output header.
