---
status: complete
priority: p1
issue_id: "011"
tags: [lasm, benchmark, postgres, workbench, scaling]
dependencies: []
---

# Problem Statement

`wb-tasks-list` was the remaining canonical workbench endpoint that still failed the strict benchmark gate under the DB-backed LASM workload.

# Findings

- Replacing the `items_blob` string-aggregation path with native JSON rows materially improved the endpoint.
- Fixed cluster mode still left a small timeout tail at target `1500 rps`.
- Proxy cluster mode cleared the remaining timeout tail on the same workload.

# Proposed Solutions

## Option 1

Lock the canonical `wb-tasks-list` benchmark baseline to LASM proxy-cluster mode and use that for the next cross-runtime rerun.

Pros:
- Matches measured behavior.
- Unblocks the benchmark queue immediately.

Cons:
- Does not yet reduce the remaining SQL/response-path cost itself.

# Recommended Action

Use proxy-cluster mode as the canonical LASM topology for `wb-tasks-list` in the next rerun and record the measured evidence in docs/artifacts.

# Acceptance Criteria

- `sec4-lasm` `wb-tasks-list` benchmark passes at target `1500 rps` with `0` non-2xx and `0` socket timeouts.
- The chosen LASM topology is recorded in the branch work log/docs.

# Work Log

### 2026-03-07 - Created task

**By:** Codex

**Actions:**
- Recorded the remaining endpoint-specific closeout task after validating proxy-cluster mode on the canonical Postgres-backed workload.

### 2026-03-07 - Completed

**By:** Codex

**Actions:**
- Ran the canonical Postgres-backed `wb-tasks-list` benchmark in LASM proxy-cluster mode.
- Confirmed `targetRps=1500`, `requestsPerSec=1499.27`, `non2xxOr3xxResponses=0`, and zero socket timeouts.
- Locked this topology as the stable baseline for the next cross-runtime rerun.
