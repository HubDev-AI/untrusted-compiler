# 1009 M39 Slice: Benchmark RSS Memory Baseline Visibility

## What It Is

This slice adds deterministic memory-signal plumbing to benchmark artifacts:

- profile summaries now include `memory.rssKb` + `memory.sampleSource`,
- compare-report and compare-matrix rows now carry `rssKb`,
- published markdown report surfaces leader RSS values and warns when memory samples are missing.

## Why It Exists

M39 post-default-backend order requires LASM stability/load hardening with baseline visibility for throughput, latency, and memory.

Before this slice, throughput/latency were first-class in benchmark outputs, but memory was not propagated through the same report contracts.

## How It Works Internally

1. Runtime/profile capture:
   - `run_profile.sh` now accepts optional service PID context through `BENCH_SERVER_PID`.
   - after each run, it samples RSS via `ps -o rss=` and passes result metadata to `wrk2_summary.sh`.

2. Summary schema/output:
   - `wrk2_summary.sh` now writes:
     - `memory.rssKb` (`number|null`),
     - `memory.sampleSource` (`ps` or `unavailable`).
   - `summary.schema.json` now requires `memory` with `rssKb`/`sampleSource`.

3. Matrix/report propagation:
   - `compare_reports.sh` and `compare_matrix.sh` now emit row-level `rssKb`.
   - compare schemas now require `rssKb` on each compare row.
   - `publish_report.sh` now renders RSS in evidence/leader tables and marks evidence quality `WARN` when memory coverage is incomplete.

4. Orchestrator wiring:
   - fixed-target and step orchestrators now pass live service PID into profile runs so RSS sampling is automatic in matrix workflows.

5. Contract/testing updates:
   - benchmark contract spec updated with memory keys,
   - script/schema contract tests and sample fixtures updated to lock new fields and invariants.

## Inputs / Outputs and Constraints

Inputs:
- benchmark profile/matrix runs with or without `BENCH_SERVER_PID`.

Outputs:
- summary artifacts with explicit memory object,
- compare artifacts with `rssKb` on each row,
- markdown report with memory coverage + leader RSS display.

Constraints:
- if no PID is provided (or sampling fails), `rssKb` stays `null` and `sampleSource` is `unavailable`,
- RSS sampling is point-in-time per completed profile run (not peak-process tracking).

## Failure Modes and Diagnostics

- Invalid optional RSS override argument in `wrk2_summary.sh` exits with deterministic usage error.
- Missing memory contract keys in fixtures/artifacts are caught by updated schema/contract tests.
- Missing runtime memory samples are surfaced as report `WARN` with explicit coverage counts.

## Example Usage

Matrix run with automatic RSS capture:

```bash
make -C benchmark-suite bench-matrix
```

Standalone profile with explicit PID sampling:

```bash
BENCH_SERVER_PID=12345 make -C benchmark-suite bench-profile IMPL=sec4-lasm ENDPOINT=ping
```

## Tradeoffs and Next Steps

Tradeoffs:
- current memory signal is lightweight and deterministic, but not a peak-RSS time series.

Next steps:
1. add explicit memory regression thresholds in benchmark gates,
2. add peak RSS capture mode for long-duration/step-load phases where needed.
