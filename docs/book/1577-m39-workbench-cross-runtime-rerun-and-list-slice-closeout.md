# M39: Workbench Cross-Runtime Rerun and List Slice Closeout

Date: `2026-03-07`

## What changed

- Re-ran the canonical six-endpoint DB-backed workbench benchmark matrix on:
  - `sec4-lasm`
  - `node`
  - `go`
  - `rust`
- Used the stable mixed-workload LASM publication topology:
  - `--lasm-mode single`
- Refreshed the canonical artifact family:
  - `benchmark-suite/results/workbench-benchmark-report.md`
  - `benchmark-suite/results/workbench-benchmark-report.html`
  - `benchmark-suite/results/summaries/workbench-benchmark-runs.json`
  - `benchmark-suite/results/summaries/workbench-benchmark-compare-matrix.json`
  - `benchmark-suite/results/summaries/workbench-benchmark-analysis.json`
- Closed the stale `wb-tasks-list` blocker wording and normalized the publication topology notes in roadmap/handoff docs.

## Why

Earlier alpha-closeout memory still referenced an older benchmark publication run that had impl-level failures and stale topology assumptions. After the `wb-tasks-list` stabilization work, that was no longer the right canonical story.

The closeout needed one fresh same-contract rerun that:

1. used the same six-endpoint Postgres-backed workload across all four implementations,
2. finished cleanly,
3. produced one canonical report family,
4. replaced the older "still failing" wording in the repo memory.

## Final evidence

Run window:

- `startedAt`: `2026-03-07T18:29:46Z`
- `finishedAt`: `2026-03-07T18:38:37Z`
- totals: `passed=4 failed=0 skipped=0`

Published winners:

- `wb-tasks-post`: `sec4-lasm 491.81 req/s`, `p99 20.48ms`
- `wb-tasks-with-comment`: `sec4-lasm 198.00 req/s`, `p99 28.41ms`
- `wb-tasks-with-comment-tx`: `sec4-lasm 198.13 req/s`, `p99 28.62ms`
- `wb-task-comment-post`: `sec4-lasm 491.79 req/s`, `p99 16.88ms`
- `wb-task-get`: `sec4-lasm 2465.50 req/s`, `p99 7.38ms`
- `wb-tasks-list`: `sec4-lasm 1458.07 req/s`, `p99 12.76ms`

Cross-runtime ranking stayed consistent across the full workload:

1. `sec4-lasm`
2. `go`
3. `rust`
4. `node`

## Consequence

The alpha benchmark-closeout story is now simple:

- the canonical mixed-workload publication run is clean,
- `sec4-lasm` leads all six endpoints,
- the old `wb-tasks-list` blocker note is closed,
- post-alpha work moves to scaling/runtime tuning from this clean baseline, not back to alpha-proof repair work.
