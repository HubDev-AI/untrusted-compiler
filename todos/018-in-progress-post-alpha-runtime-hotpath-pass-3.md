---
status: completed
priority: p1
issue_id: "018"
tags: [post-alpha, lasm, postgres, runtime, hotpath]
dependencies: ["017"]
---

# Post-Alpha Runtime Hotpath Pass 3

This board tracks the next implementation-first slice after `017`.
Focus: LASM + Postgres runtime hot-path reductions and publishable status sync.

# Task Board

- [x] T1: Execute Postgres runtime hot-path optimization batch
  - Goal: reduce avoidable allocations and key recomputation in DB execution and persist paths.
  - Monitor: `compiler/sec4-cli/src/lasm_db_runtime_postgres.rs`, `compiler/sec4-cli/src/lasm_db_runtime_postgres_persist.rs`, `compiler/sec4-cli/src/lasm_db_client/operations.rs`
- [x] T2: Close docs/status sync for this batch
  - Goal: record the optimization slice in handoff + roadmap + book index/chapter.
  - Monitor: `docs/codex-operator-handoff.md`, `docs/05-sec4-master-roadmap.md`, `docs/book/README.md`
- [x] T3: Run focused LASM benchmark sanity on canonical endpoints
  - Goal: verify no functional regression on the hot-path-changed routes.
  - Monitor: `benchmark-suite/results/summaries/sec4-lasm-*.json`
- [x] T4: Publish merge-ready summary
  - Goal: provide exact changed files, validation commands, and next task pick.
  - Monitor: operator update + commit list

# Work Log

### 2026-03-10 - Board created

**By:** Codex

**Actions:**
- Created next execution board and marked hot-path code batch as complete (`T1`).
- Remaining scope for this board is docs sync + focused LASM benchmark sanity + merge summary.

### 2026-03-10 - T2 completed

**By:** Codex

**Actions:**
- Synced this batch in:
  - `docs/codex-operator-handoff.md`
  - `docs/05-sec4-master-roadmap.md`
  - `docs/book/README.md`
- Added chapter:
  - `docs/book/1579-m39-lasm-postgres-hotpath-key-caching-and-persist-queue-reuse.md`

### 2026-03-10 - T3 completed (focused LASM benchmark sanity)

**By:** Codex

**Commands:**
- `BENCH_WORKBENCH_REQUIRE_WRK2=1 BENCH_DURATION=10s benchmark-suite/scripts/run_workbench_benchmark_matrix_local.sh --impls sec4-lasm --endpoints wb-task-get,wb-tasks-list,wb-tasks-with-comment --fail-on-impl-failure 1 --lasm-mode single --profile-retry-on-failure 0 --out-runs <tmp> --out-compare <tmp> --out-analysis <tmp> --out-report <tmp> --out-report-html <tmp>`

**Results:**
- totals: `passed=1 failed=0 skipped=0`
- `wb-task-get`: `2428.63 req/s`, `p99 3.06ms`
- `wb-tasks-list`: `1458.19 req/s`, `p99 4.68ms`
- `wb-tasks-with-comment`: `193.13 req/s` (target `200`), run passed

### 2026-03-10 - T4 completed

**By:** Codex

**Actions:**
- Published merge-ready task-board status with concrete runtime/code/docs commits.
- Marked board `018` task set complete.
