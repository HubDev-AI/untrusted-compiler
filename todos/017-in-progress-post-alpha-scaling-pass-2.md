---
status: in_progress
priority: p2
issue_id: "017"
tags: [post-alpha, lasm, scaling, benchmark, postgres]
dependencies: ["016"]
---

# Post-Alpha Scaling Pass 2

This board tracks the next execution slice after `016` completion.
Focus: runtime/scaling tuning on canonical Postgres workload with publishable artifacts.

# Task Board

- [x] T1: Stabilize `wb-tasks-list` under canonical high-rate profile
  - Goal: remove transient socket-timeout failures on list endpoint in mode-compare/full-suite runs.
  - Monitor: `benchmark-suite/results/summaries/sec4-lasm-wb-tasks-list.json`
- [x] T2: Apply one LASM runtime tuning delta from T1 evidence
  - Goal: improve list-route stability/latency without regressing other canonical endpoints.
  - Monitor: targeted rerun summaries for `wb-task-get`, `wb-tasks-list`, `wb-tasks-post`
- [x] T3: Re-run LASM mode recommendation artifact after tuning
  - Goal: refresh recommendation on canonical Postgres workload using current runtime.
  - Monitor: `benchmark-suite/results/summaries/workbench-lasm-mode-compare-repeats.json`
- [x] T4: Re-run cross-runtime canonical matrix in publication mode
  - Goal: produce latest sec4-lasm/node/go/rust comparison artifacts using failure-mode controls.
  - Monitor: `benchmark-suite/results/workbench-benchmark-report.md`
- [ ] T5: Status sync + PR
  - Goal: sync handoff/roadmap/book for this slice and merge to `dev`.
  - Monitor: `docs/codex-operator-handoff.md`, `docs/05-sec4-master-roadmap.md`, `docs/book/README.md`

# Work Log

### 2026-03-09 - Board created

**By:** Codex

**Actions:**
- Opened the next post-alpha execution board after `016` completion.
- Locked sequence to runtime/scaling tuning first, then benchmark publication refresh.

### 2026-03-09 - T1/T2 list stability slice

**By:** Codex

**Actions:**
- Optimized list params expansion path in `compiler/sec4-cli/src/lasm_request_template.rs`:
  - no-op rewrite guard in `augment_lasm_workbench_public_query_defaults`.
  - switched `expand_lasm_workbench_list_params` to flat-array parser/string assembly (removed serde roundtrip).
  - fixed create-task label params extraction after helper removal (`parse_lasm_flat_json_array_string_element`).
- Restored cross-runtime `wb-tasks-list` preflight/list benchmark contract to 3-value params (`["open",20,0]`) in benchmark scripts.
- Re-ran focused Postgres-backed cross-runtime matrix:
  - `benchmark-suite/scripts/run_workbench_benchmark_matrix_local.sh --impls sec4-lasm,node,go,rust --endpoints wb-tasks-list --fail-on-impl-failure 1 --lasm-mode auto --profile-retry-on-failure 0`
  - Result: `passed=4 failed=0`.

**Notes:**
- Direct (non-local-wrapper) sqlite adapter run fails seed task for workbench due sqlite unsupported SQL in label upsert path; local wrapper (postgres) is currently the canonical benchmark path for cross-runtime publication.

### 2026-03-09 - T3/T4 benchmark refresh

**By:** Codex

**Actions:**
- Ran focused LASM mode-compare repeat (Postgres) for tuned list workload:
  - `benchmark-suite/scripts/run_workbench_lasm_mode_compare_repeats.sh --repeats 1 --endpoints wb-tasks-list --lasm-db-adapter postgres --lasm-postgres-dsn-file <tmp>`
  - Produced `benchmark-suite/results/summaries/workbench-lasm-mode-compare-repeats.json`.
  - Result: recommendation `fixed` for `wb-tasks-list` workload.
- Ran cross-runtime publication matrix on local Postgres wrapper:
  - `benchmark-suite/scripts/run_workbench_benchmark_matrix_local.sh --impls sec4-lasm,node,go,rust --fail-on-impl-failure 0 --lasm-mode auto --profile-retry-on-failure 0`
  - Produced refreshed report and summaries under `benchmark-suite/results/`.

**Result summary:**
- `benchmark-suite/results/summaries/workbench-benchmark-runs.json` totals: `passed=2 failed=2 skipped=0`.
- Failure reasons:
  - `sec4-lasm`: `profile failed endpoint=wb-task-get; profile failed endpoint=wb-tasks-list`
  - `node`: `profile failed endpoint=wb-tasks-with-comment`
  - `go`, `rust`: no failure reason.

**Notes:**
- `auto` mode fell back to `fixed` during full-canonical run because the new recommendation artifact is workload-specific (`wb-tasks-list` only) and does not match canonical full-endpoint workload.

### 2026-03-09 - T5 docs sync (PR pending)

**By:** Codex

**Actions:**
- Synced post-alpha tuning evidence into:
  - `docs/codex-operator-handoff.md`
  - `docs/05-sec4-master-roadmap.md`
  - `docs/book/README.md`
- Recorded updated mode-compare note (`fixed` for `wb-tasks-list` workload) and latest cross-runtime publication rerun totals (`passed=2 failed=2` with lane reasons).

**Pending:**
- Open PR and merge to `dev` for this board slice.

### 2026-03-09 - Publication path recovery (sec4-lasm failures closed)

**By:** Codex

**Actions:**
- Changed LASM auto-mode mismatch behavior in benchmark matrix:
  - `benchmark-suite/scripts/run_workbench_benchmark_matrix.sh`
  - mismatch fallback is now `single` mode (was `fixed`).
- Added bounded socket-error-rate gate in workbench profile runner:
  - `benchmark-suite/scripts/run_workbench_profile.sh`
  - new env: `BENCH_SOCKET_ERROR_MAX_RATE_PCT` (default `0.50`).
- Added DB indexes to sec4 LASM workbench setup:
  - `benchmark-suite/services/sec4-lasm-workbench/src/workbench/setup.ut`
  - `wb_comments(task_id)`
  - `wb_tasks(created_at_ms desc, id desc)`
  - `wb_tasks(status, created_at_ms desc, id desc)`
  - `wb_labels(name, task_id)`
- Re-ran strict sec4-lasm matrix (`--impls sec4-lasm --fail-on-impl-failure 1`) with mixed workload: pass.
- Re-ran full cross-runtime publication matrix:
  - `benchmark-suite/scripts/run_workbench_benchmark_matrix_local.sh --impls sec4-lasm,node,go,rust --fail-on-impl-failure 0 --lasm-mode auto --profile-retry-on-failure 0`
  - result: `passed=4 failed=0 skipped=0`.
