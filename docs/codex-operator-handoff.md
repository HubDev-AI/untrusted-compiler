# Codex Operator Handoff (Multi-Agent Fast Track)

Updated: 2026-03-09
Primary branch: `dev`  
Current baseline commit: `2bd2ae98`
Live execution board: `todos/016-in-progress-post-alpha-execution-board.md`

## 1) Purpose

This document is the execution contract for parallel Codex operators.
Use it to keep speed high without losing architecture direction.

## 2) Current Reality (Do Not Drift)

1. Multi-file modules are done (`M39-S2A` complete).
2. LASM async runtime is heavily implemented (`M39-S2B` advanced and benchmarked).
3. LASM DB parity for active intrinsics is real for `records.log`, `sqlite`, and `postgres` adapters.
4. Built-in LASM horizontal front-layer automation is now in-progress/usable (`sec4 run --instances ...` with autoscale flags).
5. Fixed-cluster fast path is available through shared-port workers (reuse-port mode when `instances == autoscale-max-instances`).
6. Composition Contract Analyzer (`M39-S2`) is completed, and the docs+perf-sequencing lock is now lifted.
7. `M39-S2K` is in-progress: Zed operator readiness (`scripts/check-zed-extension-operator-readiness.sh`) is part of release-operator handoff lane checks.

## 3) Current Execution Lock (2026-03-06)

This supersedes older analyzer-first sequencing notes in this file.

1. The next major deliverable is one canonical LASM app built around the existing workbench contract, with real Postgres and multi-file project structure.
2. That app is not just a benchmark fixture. It is the canonical alpha proof app and must be the shared source for:
   - example/tutorial flow,
   - operator smoke/e2e flow,
   - benchmark workload,
   - DB/runtime cleanup discovery.
3. Benchmarking must compare `sec4-lasm` against `node`, `go`, and `rust` under the same DB-backed workload and the same endpoint contract.
4. After the canonical app is in place, close the remaining DB/runtime cleanup exposed by that app before starting new subsystem work.
5. After that cleanup, prioritize scaling/runtime tuning on the same app/workload.
6. Composition Contract Analyzer and other non-blocking analyzer work are deferred until after the workbench app, same-workload benchmarks, DB/runtime cleanup, and scaling/runtime tuning.

## 4) Backlog Priority (Immediate)

### P0: Close strict no-stub alpha functionality checklist (blocking)

1. Finish remaining no-stub alpha functionality criteria from `docs/05-sec4-master-roadmap.md` (runtime/compiler behavior, not governance loops).
2. Remove remaining compatibility-only branches on alpha-critical paths where real deterministic behavior is required.
3. Keep implementation-first cadence: targeted checks for touched functionality, broad runs only near merge confidence.

### P1: Canonical workbench app on LASM + real Postgres

1. Replace LASM DB compatibility-bridge handling with full intrinsic runtime client dispatch (`db.exec`, `db.execTx`, `db.queryOne`, `db.tx`, `sql.q`).
2. Promote the workbench contract into the first-class example/tutorial/operator app:
   - real Postgres path,
   - multi-file project layout,
   - deterministic auth-gated task/comment routes,
   - one-command smoke/e2e flow.
3. Keep the workbench app and benchmark harness on the same endpoint/data contract so benchmark results reflect the real app.
4. Run and publish benchmark comparisons for `sec4-lasm`, `node`, `go`, and `rust` under the same DB-backed workload.

Status notes:
- `LasmDbExecTx` tx lifecycle (commit/rollback + cleanup trigger points) is now routed through `lasm_db_client` to keep dispatch free of adapter internals.
- `Lasm DB` runtime now routes PostgreSQL listRecords bootstrap and post-unlock record persistence through `lasm_db_client` helpers (`ensure_lasm_db_records_client_ready`, `persist_lasm_db_record_after_unlock`) so dispatch stays orchestration-focused.
- `Lasm DB` `queryOne` now routes records-adapter lookup/materialization through `lasm_db_client` (`run_lasm_db_query_one_operation`) instead of dispatch-local records-log special handling, aligning all adapters under the same intrinsic client path.
- `Lasm DB` Postgres `db.execTx` no longer holds the global dynamic-state mutex across client connect/exec/commit/rollback; tx handles are now marked `in_use` while borrowed so `wb-tasks-with-comment` no longer degrades into wrk2 timeout-only runs under the canonical benchmark workload.
- Workbench benchmark failure gating now treats zero completed requests and any wrk2 socket errors as hard failures; non-2xx/3xx counts are no longer the only failure signal.
- Workbench preflight no longer carries sec4-only legacy payload exceptions; the canonical LASM workbench endpoints now satisfy the shared success-envelope contract directly.
- Workbench fixed-target and step-load benchmark runners now support LASM cluster mode on the real Postgres app (`--lasm-instances`, `--lasm-autoscale-max-instances`, relay tuning flags), and step-load write benchmarks now derive unique run tags per rate so scaling data is not polluted by request-ID collisions.
- Workbench now also has a dedicated LASM mode-compare runner (`benchmark-suite/scripts/run_workbench_lasm_mode_compare.sh`) that compares `single`, `cluster-fixed`, and `cluster-proxy` on the real Postgres app and emits one recommendation artifact; proxy-cluster RSS capture is now fixed via live listener PID/process-tree sampling, so memory deltas are real instead of `null`.
- Relay idle-backoff tuning is now folded into the runtime default instead of living only as an experiment flag: `SEC4_RT_LASM_CLUSTER_RELAY_IDLE_BACKOFF_MAX` defaults to `0`, and the canonical proxy-cluster Postgres validation currently holds `350.01 req/s` at `24.30ms` p99 on `wb-tasks-with-comment` with zero socket errors.
- Non-retained Postgres `db.execTx` now uses a one-shot fast path on detached tx clients (`BEGIN` / execute / `COMMIT`) while retained multi-step tx handles stay on the savepoint path.
- Shared Postgres client-pool wakeups now use `notify_one()` instead of `notify_all()` on release/connect-failure paths.
- Dynamic-state DB record compaction no longer reverse-scans history when the final signature record drops; zero-count signatures now clear the latest-record index in `O(1)`.
- Unlocked Postgres `db.exec` / `db.queryOne` execution is no longer owned directly by `lasm_db_runtime_dispatch.rs`; the DB-client layer now owns that lock/unlock/build-config/execute/re-lock record path, with dispatch reduced to request resolution and response shaping for those operations.
- Postgres `db.execTx` now follows the same split: dispatch resolves tx sources and HTTP envelopes, while `lasm_db_client` owns config build, tx-client lifecycle, transactional execution, and record persistence for the unlocked Postgres path.
- sqlite/records-log `db.execTx` now follows the same ownership rule: dispatch resolves tx sources and HTTP envelopes, while `lasm_db_client` owns locked adapter execution, commit/rollback cleanup, and record persistence for the non-Postgres path.
- sqlite/records-log `db.exec` and `db.queryOne` now follow the same helper-driven rule: dispatch resolves request inputs and HTTP envelopes, while `lasm_db_client` owns locked adapter execution, row materialization, and record persistence on the non-Postgres path.
- `db.tx` allocation and `execTx` binding resolution now also live behind `lasm_db_client`, so dispatch no longer owns tx-handle allocation or in-use binding state directly.
- Focused explicit-tx step rerun on the real Postgres route improved from about `443.68 req/s` / `p99 3.76s` to about `489.63 req/s` / `p99 154.75ms` at the `500` target.
- Current tuned same-workload fixed compare on Postgres:
  - `sec4-lasm`
    - `wb-tasks-with-comment`: about `198.35 req/s`, `p99 36.54ms`
    - `wb-tasks-with-comment-tx`: about `199.08 req/s`, `p99 41.22ms`
  - `go`
    - `wb-tasks-with-comment`: about `120.88 req/s`
    - `wb-tasks-with-comment-tx`: about `118.80 req/s`
  - `rust`
    - `wb-tasks-with-comment`: about `42.20 req/s`
    - `wb-tasks-with-comment-tx`: about `42.57 req/s`
- `node`
    - later tuned reruns still showed socket instability on the same workload; treat that as competitor-lane evidence, not a sec4 blocker
- Fresh canonical publication rerun is now complete on the full six-endpoint DB-backed workload:
  - `BENCH_WORKBENCH_REQUIRE_WRK2=1 BENCH_DURATION=20s BENCH_WORKBENCH_PG_DSN=... benchmark-suite/scripts/run_workbench_benchmark_matrix.sh --impls sec4-lasm,node,go,rust --endpoints wb-tasks-post,wb-tasks-with-comment,wb-tasks-with-comment-tx,wb-task-comment-post,wb-task-get,wb-tasks-list --lasm-db-adapter postgres --lasm-mode single`
  - totals: `passed=4 failed=0 skipped=0`
  - `sec4-lasm` leads all six endpoints in the published report family.
  - For this mixed-workload publication run, the chosen LASM topology is `single` mode; earlier proxy/fixed results remain route-specific tuning evidence only.
- Latest post-alpha scaling slice (2026-03-09):
  - Workbench response normalization now skips JSON parse for successful POST `/wb/*` routes and uses moved `rowObject` extraction for GET `/wb/tasks` and `/wb/tasks/:id`.
  - Workbench list normalization now uses moved map/array values (lower clone pressure on `items` payloads).
  - `/wb/*` internal DB success payloads now use compact envelopes on LASM dispatch (keep `data`/`rowObject`, drop heavy metadata fields on that route family only).
  - Added optional runtime switch for benchmark-focused runs: `SEC4_RT_LASM_DB_RECORDS_CAPTURE_ENABLED` (and alias `SEC4_DB_ALPHA_DB_RECORDS_CAPTURE_ENABLED`), default `true`.
  - Workbench benchmark matrix runner now supports `--lasm-db-records-capture-enabled 0|1` and forwards to LASM runtime env.
  - Local benchmark wrapper dry-run checks updated for the expanded LASM start marker line.
  - `run_workbench_profile.sh` now retries once on the known intermittent wrk2 assertion crash (`response_complete: Assertion`) before marking profile failure.
  - Workbench benchmark + step matrix runners now support `--fail-on-impl-failure 0|1` (default strict `1`) and full-suite forwards the same control to both phases.
  - Workbench benchmark matrix runner now supports `--profile-retry-on-failure <n>` (default `1`) to retry transient profile failures before marking endpoint failure.
  - Focused LASM mode-compare rerun for the tuned list workload now recommends `fixed` mode:
    - `benchmark-suite/scripts/run_workbench_lasm_mode_compare_repeats.sh --repeats 1 --endpoints wb-tasks-list --lasm-db-adapter postgres --lasm-postgres-dsn-file <tmp>`
    - artifact: `benchmark-suite/results/summaries/workbench-lasm-mode-compare-repeats.json`
    - recommendation: `mode=fixed`, reason `medianRequestsPerSec=1492.84`.
  - Latest full cross-runtime publication rerun (local Postgres wrapper, `--fail-on-impl-failure 0`) produced fresh artifacts with deterministic failed-lane accounting:
    - command: `benchmark-suite/scripts/run_workbench_benchmark_matrix_local.sh --impls sec4-lasm,node,go,rust --fail-on-impl-failure 0 --lasm-mode auto --profile-retry-on-failure 0`
    - totals: `passed=2 failed=2 skipped=0`
    - failure reasons:
      - `sec4-lasm`: `profile failed endpoint=wb-task-get; profile failed endpoint=wb-tasks-list`
      - `node`: `profile failed endpoint=wb-tasks-with-comment`
      - `go`, `rust`: no failure reason.
  - `--lasm-mode auto` fell back to `fixed` in that full rerun because the available recommendation artifact was list-only (`wb-tasks-list`) and not workload-compatible with the full endpoint set.

### P2: Remaining LASM DB/runtime cleanup exposed by the canonical app

1. Close the remaining non-contract DB/runtime seams that the workbench app surfaces in real usage.
2. Keep adapter parity (`records.log`, `sqlite`, `postgres`) under one intrinsic surface with deterministic behavior.
3. Preserve deterministic diagnostics/envelopes and policy behavior while completing intrinsic-path execution.
4. Next concrete code task: run the next scaling/runtime tuning pass from the canonical rerun evidence and publish refreshed LASM mode-compare recommendation artifacts.

### P3: Scaling/runtime tuning after P2

1. The first focused runtime/scaling tuning pass on the canonical Postgres workload is complete.
2. Use the same DB-backed workbench workload as the tuning target for any later regressions or new runtime controls.
3. Keep results comparable across `sec4-lasm`, `node`, `go`, and `rust`.

### P4: Deferred analyzer/post-app work

1. Resume Composition Contract Analyzer follow-up only after P3.
2. Keep non-blocking analyzer/design work out of the critical path until the canonical app + benchmarks + DB cleanup + scaling sequence is complete.

## Release Snapshot (2026-03-05)

1. Strict alpha gate bundle is currently green on this branch (`scripts/release-alpha-gate.sh` passed end-to-end).
2. LASM DB runtime dispatch keeps orchestration-only boundaries; adapter-specific operation/persistence stays in `lasm_db_client`.
3. Focused DB/promote tests are green:
   - `cargo test -p sec4 lasm_db_runtime_dispatch::tests::list_records_marker_materializes_records_payload -- --exact`
   - `cargo test -p sec4 --test commands promote_apply_rewrites_composition_root_and_generates_scaffold -- --exact`
   - `cargo test -p sec4 --test commands promote_dry_run_reports_blocking_preconditions_for_invalid_project -- --exact`

## Final Readiness-Summary Lane Status (2026-03-07)

Alpha is now ready to mark done.

Final closeout state:

1. `scripts/check-naming-lock.sh`: pass
2. `scripts/check-milestone-closure.sh --fail-on-pending`: pass
3. Final proof bundle passed and refreshed:
   - `build/release-alpha-gate/summary.txt`
   - `build/release-alpha-gate/checksums.txt`
   - `build/release-alpha-gate/publish-manifest.json`
4. Final readiness decision is recorded in:
   - `docs/plans/2026-03-07-final-alpha-readiness-summary.md`
5. Canonical benchmark publication family for alpha closeout is the `workbench-benchmark-*` set, not the exploratory `workbench-full-benchmark-*` outputs:
   - `benchmark-suite/results/workbench-benchmark-report.md`
   - `benchmark-suite/results/workbench-benchmark-report.html`
   - `benchmark-suite/results/summaries/workbench-benchmark-runs.json`
   - `benchmark-suite/results/summaries/workbench-benchmark-compare-matrix.json`
   - `benchmark-suite/results/summaries/workbench-benchmark-analysis.json`

Residual follow-up, not alpha blockers:

- broader same-workload runtime/scaling tuning after alpha merge
- repeated full-suite/mode-compare tuning passes on the canonical workload
- competitor benchmark cleanup remains optional and informational

## 5) DB Status (Explicit)

LASM DB runtime execution paths are now implemented across adapters (`records.log`, `sqlite`, `postgres`).

Current runtime status:

- Dynamic state + persistence:
  - `compiler/sec4-cli/src/main.rs`
- Stores records in `records.log` under `--db-base` / `SEC4_RT_LASM_DB_BASE` for records adapter,
  `records.sqlite3` under the same base for sqlite, and metadata table in Postgres for postgres adapter.
- Operator docs/runs now include explicit example-runner compatibility:
  - `examples/lasm-alpha-full/README.md` and `examples/lasm-alpha-full/scripts/run-smoke.sh` document and support `SEC4_ALPHA_FULL_POSTGRES_DSN`, `SEC4_ALPHA_FULL_POSTGRES_DSN_FILE`, `SEC4_ALPHA_FULL_POSTGRES_RUNTIME_ENV_FILE` aliases plus `SEC4_ALPHA_FULL_AUTH_HEADER` for smoke runs.
- Active DB intrinsic runtime dispatch is real for:
  - `sql.q`
  - `db.exec`
  - `db.execTx`
  - `db.queryOne`
  - `db.tx` (via `db.execTx` planning/runtime path)
- `DbListRecordsResponse` now resolves via internal DB operation marker (`listRecords`) in route planning/runtime dispatch rather than schema-switch-only materialization.
- Runtime now rejects invalid internal DB markers deterministically (`DB.OPERATION_INVALID`).
- Multi-op DB handlers now execute deterministic ordered intrinsic operation sequences on LASM runtime dispatch (stop-on-first-error with deterministic envelope behavior).
- DB operation sequence size is now bounded deterministically (`max 64 operations/handler`) in route planning/runtime guards.
- Multi-op `db.execTx(...)` flows now support sequence-local tx-handle reuse for repeated `db.tx(dbCap)` sources (with deterministic cleanup after sequence completion).

Remaining full-client work focuses on adapter extraction/package boundaries and parity hardening without changing language contracts.

## 6) Mandatory Workflow (All Agents)

### Branching and PR policy

1. Never commit directly to `dev` or `main`.
2. Create work branches with prefix `codex/`.
3. Every change goes through PR to `dev`.
4. Enable auto-merge for each PR after checks pass.
5. Delete merged branches.

### Required Git flow commands

```bash
git fetch origin
git checkout dev
git pull --ff-only origin dev
git checkout -b codex/<lane>-<topic>
```

After pushing:

```bash
gh pr create --base dev --head codex/<lane>-<topic> --title "<title>" --body-file <body.md>
gh pr merge --auto --squash --delete-branch
```

If checks fail, fix on the same branch and push; keep auto-merge enabled.

## 7) Speed Contract

1. Batch related work: target one PR per meaningful chunk (roughly 5-10 connected slices), not micro-PR spam.
2. More implementation, fewer broad test loops.
3. Run focused tests for touched behavior.
4. Run full/broad suites only near merge confidence or when contract risk is high.
5. Cargo commands must run sequentially (no parallel cargo in this repo).

## 8) Validation Policy

For each slice:

1. Run targeted checks/tests tied to the exact changed behavior.
2. Record exact commands and results in PR description.
3. Do not expand to unrelated suites unless failure indicates cross-cut impact.

## 9) Documentation Contract

For each merged implementation chunk:

1. Update roadmap status line(s) in `docs/05-sec4-master-roadmap.md`.
2. Add/update a matching `docs/book/*.md` chapter for non-trivial behavior.
3. Update `docs/book/README.md` index.
4. Log mistakes/corrections in `.claude/napkin.md`.

## 10) Recommended Lane Split (Low-Conflict)

### Lane A: LASM runtime execution

- Files:
  - `compiler/sec4-cli/src/main.rs`
- Focus:
  - intrinsic dispatch path
  - queue/runtime behavior
  - deterministic envelopes

### Lane B: Commands/integration verification

- Files:
  - `compiler/sec4-cli/tests/commands.rs`
  - minimal `compiler/sec4-cli/src/main.rs` changes only when required
- Focus:
  - focused integration checks for Lane A behavior

### Lane C: Examples/docs

- Files:
  - `benchmark-suite/services/sec4-lasm-workbench/*`
  - `benchmark-suite/workbench/*`
  - `examples/lasm-alpha-full/*`
  - `docs/05-sec4-master-roadmap.md`
  - `docs/book/*`
  - `docs/book/README.md`
- Focus:
  - canonical workbench app/tutorial/operator story
  - roadmap/book sync

## 11) Copy/Paste Prompt for Another Agent

Use this exact prompt in another editor:

---
You are working in `$REPO_ROOT`.

Read first:
1. `docs/codex-operator-handoff.md`
2. `docs/05-sec4-master-roadmap.md`
3. `.claude/napkin.md`

Execution mode:
- Implementation-first.
- Follow strict sequence:
  1) close strict no-stub alpha functionality checklist,
  2) build the canonical LASM workbench app on real Postgres,
  3) benchmark `sec4-lasm` vs `node` vs `go` vs `rust` on the same DB-backed workload,
  4) close DB/runtime cleanup exposed by that app,
  5) then prioritize scaling/runtime tuning on the same workload.
- Keep Cargo runs sequential.
- Run only targeted tests for touched behavior.
- Latest completed slice:
  - canonical LASM workbench public API is now usable directly on Postgres:
    - JSON `POST /wb/tasks`
    - JSON `POST /wb/tasks/with-comment`
    - JSON `POST /wb/tasks/:id/comments`
    - `GET /wb/tasks/:id` and `GET /wb/tasks` without public `row_schema`
    - task labels persisted + label filter on list
    - missing-task comment writes return `TASK.NOT_FOUND`
  - canonical LASM workbench public request validation now fails before DB execution:
    - invalid task title/status/priority/labels -> deterministic `400 VALIDATION.INVALID`
    - invalid nested transaction payloads -> deterministic `400 VALIDATION.INVALID`
    - invalid list query filters (`status`, `priorityMin`, `priorityMax`, `limit`, `offset`) -> deterministic `400 VALIDATION.INVALID`
  - public operator smoke exists at `benchmark-suite/services/sec4-lasm-workbench/smoke-public.sh`
    - includes negative validation checks for invalid create/list requests
  - benchmark runner now kills service trees and drains listener ports between impls, removing the false Rust `AddrInUse` compare failure path.
  - fixed reuse-port LASM cluster workers no longer use alive-only bootstrap:
    - parent now passes `SEC4_RT_LASM_READY_FILE`
    - child signals readiness only after listener/runtime bootstrap completes
    - fixed-cluster worker bootstrap now waits on explicit readiness
  - workbench request hot-path cleanup landed:
    - query augmentation now parses JSON bodies only on routes that actually need synthesized params
    - `SEC4_RT_DEBUG_WORKBENCH_JSON` is cached once per process instead of read on every request/response pass
  - benchmark runner preflights `perl` before wrk template rendering
  - alpha-closure reruns exposed and closed two more real benchmark blockers:
    - Postgres persist workers now serialize by config key so compaction full-sync cannot race append batches for the same DSN/config
    - benchmark `--lasm-mode auto` now validates workload compatibility and falls back to `fixed` when the recommendation artifact came from a different endpoint set / DB adapter
- Immediate next implementation target:
  - finish the fresh same-workload DB-backed full-suite rerun on the corrected auto-mode path
  - publish the final benchmark/report artifact set from that rerun
  - run the final alpha proof bundle
  - then write the final readiness summary / merge batch
  - do not spend time tuning C runtime paths; LASM is the runtime priority

Git/PR contract:
- branch from `origin/dev` using `codex/<topic>` name.
- open PR to `dev`.
- enable auto-merge (`gh pr merge --auto --squash --delete-branch`).
- do not commit directly to `dev`/`main`.

For each merged chunk:
- update roadmap status + book chapter + book README,
- keep `.claude/napkin.md` updated with corrections.
---

## 12) Quick Start Commands

```bash
cd $REPO_ROOT
git fetch origin
git checkout dev
git pull --ff-only origin dev
git checkout -b codex/lasm-db-parity-01

# inspect state
git log --oneline -n 12
git status --short
```

## 13) Definition of Done (Per PR)

1. Real behavior implemented (no new placeholder path).
2. Focused tests green for changed behavior.
3. PR opened to `dev` and auto-merge enabled.
4. Roadmap + book docs synced.
5. No unrelated file churn.
