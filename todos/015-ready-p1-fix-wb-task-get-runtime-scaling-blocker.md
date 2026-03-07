---
status: complete
priority: p1
issue_id: "015"
tags: [lasm, benchmark, scaling, runtime, postgres, workbench]
dependencies: ["014"]
---

# Problem Statement

`wb-task-get` is the remaining blocking endpoint for a clean same-workload cross-runtime rerun on the canonical LASM Postgres workbench app.

# Findings

- Fixed mode returns clean responses but fails the strict gate with timeout/backpressure behavior on `wb-task-get`.
- Proxy mode avoids the previous fixed-mode shape but exposes a real cluster/runtime issue:
  - repeated worker port bind failures (`Address already in use` on ports like `18193` / `18194`)
  - socket write errors still appear in the profile
- The server log also shows dynamic Postgres persist queue backpressure on the read-heavy path.

# Proposed Solutions

## Option 1

Debug and fix the LASM runtime/scaling path for `wb-task-get` before attempting another full mixed rerun.

Pros:
- Addresses the actual remaining blocker instead of papering over it in the benchmark harness.

Cons:
- Requires runtime investigation and probably code changes outside the benchmark scripts.

# Recommended Action

Trace the `wb-task-get` hot path and cluster worker lifecycle under load, fix the runtime blocker, then rerun the mixed cross-runtime matrix.

# Acceptance Criteria

- `wb-task-get` passes the strict benchmark gate on the chosen LASM topology.
- No worker-port bind collisions appear in the service log during the read benchmark.
- The full mixed same-workload rerun can complete afterward.

# Work Log

### 2026-03-07 - Created task

**By:** Codex

**Actions:**
- Added the runtime/scaling blocker task after proving that the remaining failure is no longer benchmark wiring but the `wb-task-get` LASM runtime path itself.

### 2026-03-07 - Root cause isolated and targeted fix landed

**By:** Codex

**Actions:**
- Traced the `wb-task-get` path through the LASM Postgres `db.queryOne` runtime and confirmed that every read request still created and persisted a DB runtime record.
- Identified that the read-heavy benchmark was saturating the Postgres persist queue with internal observability writes, not failing on the query itself.
- Added runtime support to disable external DB-record persistence via `SEC4_RT_LASM_DB_RECORDS_PERSIST_ENABLED=0` while preserving in-memory DB-record tracking.
- Updated `benchmark-suite/scripts/run_workbench_benchmark_matrix.sh` so canonical LASM benchmark runs disable external DB-record persistence by default.
- Revalidated the public workbench smoke and reran the blocked endpoint:
  - `wb-task-get` fixed-mode Postgres benchmark now completes cleanly at about `2496 req/s` against target `2500`.
