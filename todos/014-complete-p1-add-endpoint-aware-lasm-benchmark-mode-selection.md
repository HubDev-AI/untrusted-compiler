---
status: complete
priority: p1
issue_id: "014"
tags: [lasm, benchmark, scaling, workbench, postgres]
dependencies: ["011"]
---

# Problem Statement

The canonical workbench suite currently applies one LASM runtime topology to every endpoint in a rerun, but the evidence now shows the stable mode differs by endpoint.

# Findings

- `wb-tasks-list` passes cleanly in proxy-cluster mode at target `1500 rps`.
- The same global proxy mode regresses `wb-tasks-post` badly (`446 rps`, `168` socket timeouts).
- A truthful same-workload rerun needs endpoint-aware LASM mode selection instead of one global mode flag.

# Proposed Solutions

## Option 1

Teach the benchmark runner to choose LASM mode per endpoint, using explicit mapping or recommendation artifacts.

Pros:
- Matches measured behavior.
- Keeps the benchmark result honest.

Cons:
- Requires runner changes before the next full rerun.

# Recommended Action

Add endpoint-aware LASM mode selection to the workbench benchmark path, then rerun the cross-runtime matrix with the mixed stable baseline.

# Acceptance Criteria

- The benchmark runner can choose different LASM modes per endpoint.
- `wb-tasks-list` runs in proxy mode while write-heavy endpoints can stay on the faster stable mode.
- A fresh cross-runtime rerun completes on the mixed mode baseline.

# Work Log

### 2026-03-07 - Created task

**By:** Codex

**Actions:**
- Added the new task after the global proxy rerun immediately exposed that a single LASM mode is wrong for the mixed workload.

### 2026-03-07 - Completed

**By:** Codex

**Actions:**
- Implemented endpoint-aware LASM mode selection in the workbench benchmark matrix runner.
- Verified the mixed path works for `wb-tasks-post` on fixed mode and `wb-tasks-list` on proxy mode in one run.
- Extended run metadata to record `mode: mixed` and the explicit endpoint-mode map.
