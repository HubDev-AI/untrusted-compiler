# 215 M10 Slice: wrk2 Summary Pipeline

This chapter documents the second M10 slice: converting raw wrk2 output into deterministic summary JSON.

## What it is

Added benchmark helper scripts:
- `benchmark-suite/scripts/wrk2_summary.sh`
- `benchmark-suite/scripts/test_wrk2_summary.sh`

Updated benchmark runner:
- `bench-*` targets now run wrk2 with `--latency`,
- `summarize` Make target generates `results/summaries/<impl>-ping.json`.

## Why it exists

The benchmark plan requires machine-readable summaries (`summary.json`) in addition to raw load logs. This slice adds a deterministic extraction step so results can be compared and archived consistently.

## How it works internally

1. Raw wrk2 output is captured in `results/raw/*.txt`.
2. `wrk2_summary.sh` extracts:
   - `Requests/sec`,
   - latency `p50`, `p95`, `p99`, and `max`.
3. Script writes summary JSON with:
   - `impl`, `endpoint`, `targetRps`, `requestsPerSec`, `latency`.
4. `test_wrk2_summary.sh` validates parser behavior against a fixed sample wrk2 output fixture.

## Inputs, outputs, and constraints

- Inputs:
  - wrk2 raw text output (`--latency` required for percentile lines).
- Outputs:
  - summary JSON files in `benchmark-suite/results/summaries/`.
- Constraints:
  - parser expects wrk2 output format; major upstream format changes require script updates.
  - current target is ping summary generation; additional endpoint summary targets can be added next.

## Failure modes and diagnostics

- missing raw file -> script exits with usage/path error.
- missing expected wrk2 tokens -> summary fields may be empty/zero and test script fails.
- Make target paths are benchmark-suite-relative and validated via `make -C benchmark-suite test-scripts`.

## Example usage

```bash
make -C benchmark-suite bench-ping IMPL=sec4
make -C benchmark-suite summarize IMPL=sec4
cat benchmark-suite/results/summaries/sec4-ping.json
```

## Tradeoffs and next steps

- Tradeoff:
  - parser is lightweight shell/awk for portability, not a full stats parser.
- Next:
  - add decode/users summary targets,
  - add consolidated result aggregator that writes one per-run `summary.json` bundle for all endpoints.
