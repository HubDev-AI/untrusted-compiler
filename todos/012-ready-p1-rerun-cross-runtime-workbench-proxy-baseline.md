---
status: complete
priority: p1
issue_id: "012"
tags: [lasm, benchmark, node, go, rust, postgres, workbench]
dependencies: ["011", "015"]
---

# Problem Statement

The canonical same-workload DB-backed benchmark set needs a fresh rerun after the `wb-tasks-list` fix so the cross-runtime comparison reflects the new stable LASM baseline.

# Findings

- Previous cross-runtime evidence was gathered before the list-path query/materialization improvement.
- The remaining instability was isolated to `wb-tasks-list`, not the other workbench endpoints.

# Proposed Solutions

## Option 1

Rerun the full workbench benchmark suite with Postgres enabled and the currently stable LASM topology for the mixed workload.

Pros:
- Produces one coherent updated comparison set.
- Keeps the workload identical across runtimes.

Cons:
- Takes longer than a single targeted profile.

# Recommended Action

Run the full cross-runtime workbench suite on the Postgres-backed benchmark app after the remaining `wb-task-get` runtime blocker is fixed, using the conservative stable LASM mixed-workload mode (`single`) unless a workload-matching recommendation artifact says otherwise.

# Acceptance Criteria

- `sec4-lasm`, `node`, `go`, and `rust` all produce fresh result artifacts on the same DB-backed workload after the `wb-task-get` blocker is resolved.
- The compare matrix, analysis JSON, Markdown report, and HTML report are refreshed.

# Work Log

### 2026-03-07 - Created task

**By:** Codex

**Actions:**
- Added the explicit rerun task after the list-path fix proved stable in proxy-cluster mode.

### 2026-03-07 - Started

**By:** Codex

**Actions:**
- Launched the same-workload DB-backed cross-runtime workbench benchmark matrix on Postgres with LASM forced to `single` mode after proving the current workload-matching stable baseline.
- Waiting on fresh `sec4-lasm`, `node`, `go`, and `rust` artifacts before closing the slice.

### 2026-03-07 - Completed

**By:** Codex

**Actions:**
- Refreshed the full same-workload DB-backed artifact family with `sec4-lasm`, `node`, `go`, and `rust` on the same six-endpoint contract.
- Final matrix totals landed cleanly at `passed=4 failed=0 skipped=0`.
- Refreshed:
  - `benchmark-suite/results/summaries/workbench-benchmark-runs.json`
  - `benchmark-suite/results/summaries/workbench-benchmark-compare-matrix.json`
  - `benchmark-suite/results/summaries/workbench-benchmark-analysis.json`
  - `benchmark-suite/results/workbench-benchmark-report.md`
  - `benchmark-suite/results/workbench-benchmark-report.html`
