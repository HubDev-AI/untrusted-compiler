---
status: completed
priority: p1
issue_id: "033"
tags: [post-alpha, lasm, db, benchmark, scaling, tutorial]
dependencies: []
---

# Post-Alpha Finalization Board (Live)

This board is the active source of truth for all post-alpha work.
It is implementation-first and kept in sync while work is running.

## Task Board

- [x] T1: Replace stale execution board references with this live board.
  - Goal: keep one active board so progress is visible in real time.
  - Deliverables:
    - `docs/codex-operator-handoff.md` points to this board.
- [x] T2: LASM DB runtime/client hot-path cleanup (code)
  - Goal: reduce lock-heavy and duplicated finalize branches in Postgres unlocked operation paths while preserving deterministic envelopes.
  - Deliverables:
    - `compiler/sec4-cli/src/lasm_db_client/operations_postgres.rs` cleanup.
    - focused `sec4` tests for touched behavior.
- [x] T3: Canonical DB-backed benchmark rerun + publication refresh
  - Goal: refresh `sec4-lasm,node,go,rust` same-contract report set from one deterministic run command.
  - Deliverables:
    - `benchmark-suite/results/workbench-benchmark-report.md`
    - `benchmark-suite/results/workbench-benchmark-report.html`
    - `benchmark-suite/results/summaries/workbench-benchmark-runs.json`
    - `benchmark-suite/results/summaries/workbench-benchmark-compare-matrix.json`
    - `benchmark-suite/results/summaries/workbench-benchmark-analysis.json`
- [x] T4: Scaling/runtime tuning pass on canonical workload
  - Goal: apply one concrete LASM runtime tuning delta from benchmark evidence and rerun focused mode-compare.
  - Deliverables:
    - one runtime/config code delta in `compiler/sec4-cli/src/*`
    - updated mode-compare artifact in `benchmark-suite/results/summaries/`
- [x] T5: Tutorial/portal sync for real DB + benchmark flow
  - Goal: ensure docs point to canonical workbench app + exact commands that now pass.
  - Deliverables:
    - updated files under `docs/tutorial-portal/`

## Active Task

- none (board complete)

## Work Log

### 2026-03-10 - Board created

**By:** Codex

**Actions:**
- Created one consolidated post-alpha board with concrete implementation tasks and artifacts.
- Marked T2 as active immediately after board creation.

### 2026-03-10 - Completed T2 LASM Postgres tx finalization cleanup

**By:** Codex

**Actions:**
- Refactored duplicated tx-handle finalize/error release branches in:
  - `compiler/sec4-cli/src/lasm_db_client/operations_postgres.rs`
- Added dedicated helpers for:
  - config-build error tx-handle release
  - client-operation error tx-handle release
  - success finalization + retained client reinsertion
- Validation:
  - `cargo check -p sec4` -> pass
  - `cargo test -p sec4 --test commands lasm_smoke_command_materializes_db_tx_response -- --exact` -> pass

### 2026-03-10 - Completed T3 canonical DB-backed matrix rerun

**By:** Codex

**Actions:**
- Refreshed cross-backend benchmark artifacts with deterministic local Postgres wrapper:
  - `BENCH_DURATION=20s benchmark-suite/scripts/run_workbench_benchmark_matrix_local.sh --impls sec4-lasm,node,go,rust --fail-on-impl-failure 0 --lasm-mode single --profile-retry-on-failure 0`
- Published refreshed canonical outputs:
  - `benchmark-suite/results/workbench-benchmark-report.md`
  - `benchmark-suite/results/workbench-benchmark-report.html`
  - `benchmark-suite/results/summaries/workbench-benchmark-runs.json`
  - `benchmark-suite/results/summaries/workbench-benchmark-compare-matrix.json`
  - `benchmark-suite/results/summaries/workbench-benchmark-analysis.json`
- Outcome from `workbench-benchmark-runs.json`:
  - `passed=3 failed=1 skipped=0` (`node` failed endpoint stability gate; `sec4-lasm`, `go`, `rust` passed)
- Endpoint leaders from `workbench-benchmark-analysis.json`:
  - all five canonical endpoints led by `sec4-lasm`

### 2026-03-10 - Completed T4 runtime tuning delta (benchmark path)

**By:** Codex

**Actions:**
- Added LASM benchmark keep-alive tuning control in:
  - `benchmark-suite/scripts/run_workbench_benchmark_matrix.sh`
- New input surface:
  - CLI: `--lasm-max-keep-alive-requests <n>`
  - ENV: `BENCH_WORKBENCH_LASM_MAX_KEEP_ALIVE_REQUESTS` (default `4096`)
- Runtime wiring:
  - forwards `SEC4_RT_LASM_MAX_KEEP_ALIVE_REQUESTS` into sec4-lasm benchmark run environment.
- Publication visibility:
  - run metadata now includes `lasm.maxKeepAliveRequests`.
- Validation:
  - `bash benchmark-suite/scripts/test_run_workbench_benchmark_matrix.sh` -> pass

### 2026-03-10 - Completed T5 tutorial portal sync

**By:** Codex

**Actions:**
- Updated benchmark tutorial to use the canonical post-alpha matrix command + outputs:
  - `docs/tutorial-portal/05-benchmarks-capacity.md`
- Documented LASM keep-alive runtime flag for real Postgres runs:
  - `docs/tutorial-portal/03-real-db-postgres-lasm.md`
- Synced tutorial portal index with strict/relaxed benchmark modes and keep-alive tuning:
  - `docs/tutorial-portal/README.md`
