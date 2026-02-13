# 236 M10 Slice: Endpoint-Scoped Report Bundling

This chapter documents endpoint-scoped report bundling to prevent stale summaries from leaking into filtered matrix runs.

## What it is

Updated:
- `benchmark-suite/scripts/build_report.sh`
- `benchmark-suite/scripts/run_comparison_matrix.sh`
- `benchmark-suite/scripts/test_build_report.sh`
- `benchmark-suite/scripts/test_run_comparison_matrix.sh`
- `benchmark-suite/README.md`

Key changes:
- `build_report.sh` now accepts optional `endpoints_csv` input,
- when endpoint list is provided, report bundling loads only those `<impl>-<endpoint>.json` summary files,
- orchestrator now passes selected endpoint set into per-implementation report generation,
- filtered endpoint runs no longer include stale endpoint summaries from previous runs.

## Why it exists

Matrix filtering (`--endpoints ...`) is useful for focused runs, but report bundling previously globbed all implementation summaries under `results/summaries`. That could silently include stale endpoints not executed in the current run.

## How it works internally

1. Orchestrator parses selected endpoint list (`endpoints_csv`).
2. For each implementation, orchestrator runs only selected endpoint profiles.
3. Orchestrator calls `build_report.sh` with the same endpoint list.
4. `build_report.sh` requires each selected summary file to exist and uses only that set.
5. Missing endpoint summary for selected list fails report generation immediately.

## Inputs, outputs, and constraints

- Inputs:
  - `build_report.sh ... [sec_audit_json] [endpoints_csv]`
  - orchestrator-selected endpoint set.
- Outputs:
  - per-implementation report constrained to selected endpoints.
- Constraints:
  - selected endpoints must have produced summary files; missing file is an error.

## Failure modes and diagnostics

- missing selected summary -> explicit error:
  - `missing summary for impl=<impl> endpoint=<endpoint>: <path>`
- empty selected endpoint set -> orchestrator validation error before execution.

## Example usage

Direct report build with endpoint scope:

```bash
benchmark-suite/scripts/build_report.sh sec4 benchmark-suite/results benchmark-suite/results/summaries/sec4-report.json "" "ping,decode"
```

## Tradeoffs and next steps

- Tradeoff:
  - endpoint-scoped bundling is stricter and will fail if selected summaries are absent, which is preferred for reproducibility.
- Next:
  - include selected endpoint set explicitly in report metadata for downstream display/CI checks.
