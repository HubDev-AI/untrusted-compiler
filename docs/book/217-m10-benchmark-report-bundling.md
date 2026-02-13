# 217 M10 Slice: Benchmark Report Bundling

This chapter documents an M10 harness slice that bundles benchmark outputs into a single report artifact.

## What it is

Added:
- `benchmark-suite/scripts/build_report.sh`
- `benchmark-suite/scripts/test_build_report.sh`
- Make target:
  - `make -C benchmark-suite report IMPL=<impl>`

The report combines:
- environment metadata (`results/env.json` when present),
- per-endpoint summary files (`results/summaries/<impl>-*.json`),
- optional `sec.audit` JSON (script parameter).

## Why it exists

M10 requires reproducible, machine-readable benchmark outputs. Per-endpoint files are useful but fragmented; release/comparison workflows need one bundle per implementation.

## How it works internally

1. Collect summary files by impl prefix from `results/summaries/`.
2. Load env metadata from `results/env.json` if available.
3. Optionally include security audit JSON.
4. Emit report JSON:
   - `version`, `impl`, `env`, `summaries`, `secAudit`.

## Inputs, outputs, and constraints

- Inputs:
  - summary JSON files from `wrk2_summary.sh`,
  - optional env file and sec-audit file.
- Outputs:
  - bundled report JSON (default Make target writes `results/summaries/<impl>-report.json`).
- Constraints:
  - at least one summary file is required for the selected impl.
  - report bundling assumes summaries are valid JSON objects.

## Failure modes and diagnostics

- no summaries found -> script exits with explicit error.
- invalid/missing optional sec-audit path -> script exits with explicit error.
- invalid JSON payloads -> jq parse failure.

## Example usage

```bash
make -C benchmark-suite report IMPL=ailang
cat benchmark-suite/results/summaries/ailang-report.json
```

Direct script usage with sec-audit inclusion:

```bash
benchmark-suite/scripts/build_report.sh \
  ailang \
  benchmark-suite/results \
  benchmark-suite/results/summaries/ailang-report.json \
  baselines/sec-audit/default-secure-prod.hello.json
```

## Tradeoffs and next steps

- Tradeoff:
  - report schema is intentionally compact and v0.1-focused.
- Next:
  - add CPU/RSS capture integration into summary/report flow,
  - emit comparison matrix reports across multiple impl bundles.
