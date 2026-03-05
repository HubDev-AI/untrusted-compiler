# Codex Operator Handoff (Multi-Agent Fast Track)

Updated: 2026-03-05  
Primary branch: `dev`  
Current baseline commit: `2bd2ae98`

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

## 3) Backlog Priority (Immediate)

### P0: Close strict no-stub alpha functionality checklist (blocking)

1. Finish remaining no-stub alpha functionality criteria from `docs/05-sec4-master-roadmap.md` (runtime/compiler behavior, not governance loops).
2. Remove remaining compatibility-only branches on alpha-critical paths where real deterministic behavior is required.
3. Keep implementation-first cadence: targeted checks for touched functionality, broad runs only near merge confidence.

### P1: Full LASM DB client package cleanup after P0

1. Replace LASM DB compatibility-bridge handling with full intrinsic runtime client dispatch (`db.exec`, `db.execTx`, `db.queryOne`, `db.tx`, `sql.q`).
2. Keep adapter parity (`records.log`, `sqlite`, `postgres`) under one intrinsic surface with deterministic behavior.
3. Preserve deterministic diagnostics/envelopes and policy behavior while completing intrinsic-path execution.
4. Extract adapter layers into packages/modules now that runtime execution is stable.

Status notes:
- `LasmDbExecTx` tx lifecycle (commit/rollback + cleanup trigger points) is now routed through `lasm_db_client` to keep dispatch free of adapter internals.
- `Lasm DB` runtime now routes PostgreSQL listRecords bootstrap and post-unlock record persistence through `lasm_db_client` helpers (`ensure_lasm_db_records_client_ready`, `persist_lasm_db_record_after_unlock`) so dispatch stays orchestration-focused.
- `Lasm DB` `queryOne` now routes records-adapter lookup/materialization through `lasm_db_client` (`run_lasm_db_query_one_operation`) instead of dispatch-local records-log special handling, aligning all adapters under the same intrinsic client path.

### P2: Composition Contract Analyzer (`M39-S2`) after P1

1. Implement analyzer guarantees for promotion-ready composition contracts.
2. Add deterministic pass/fail fixture coverage and operator docs.

### P3: Performance tuning deferred until after P2

1. Defer proxy/runtime feature-level performance tuning (including 1M req/s optimization campaign) until P2 is completed.
2. Before P2 completion, only accept performance work that is required to preserve correctness/stability contracts.

## Release Snapshot (2026-03-05)

1. Strict alpha gate bundle is currently green on this branch (`scripts/release-alpha-gate.sh` passed end-to-end).
2. LASM DB runtime dispatch keeps orchestration-only boundaries; adapter-specific operation/persistence stays in `lasm_db_client`.
3. Focused DB/promote tests are green:
   - `cargo test -p sec4 lasm_db_runtime_dispatch::tests::list_records_marker_materializes_records_payload -- --exact`
   - `cargo test -p sec4 --test commands promote_apply_rewrites_composition_root_and_generates_scaffold -- --exact`
   - `cargo test -p sec4 --test commands promote_dry_run_reports_blocking_preconditions_for_invalid_project -- --exact`

## 4) DB Status (Explicit)

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

## 5) Mandatory Workflow (All Agents)

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

## 6) Speed Contract

1. Batch related work: target one PR per meaningful chunk (roughly 5-10 connected slices), not micro-PR spam.
2. More implementation, fewer broad test loops.
3. Run focused tests for touched behavior.
4. Run full/broad suites only near merge confidence or when contract risk is high.
5. Cargo commands must run sequentially (no parallel cargo in this repo).

## 7) Validation Policy

For each slice:

1. Run targeted checks/tests tied to the exact changed behavior.
2. Record exact commands and results in PR description.
3. Do not expand to unrelated suites unless failure indicates cross-cut impact.

## 8) Documentation Contract

For each merged implementation chunk:

1. Update roadmap status line(s) in `docs/05-sec4-master-roadmap.md`.
2. Add/update a matching `docs/book/*.md` chapter for non-trivial behavior.
3. Update `docs/book/README.md` index.
4. Log mistakes/corrections in `.claude/napkin.md`.

## 9) Recommended Lane Split (Low-Conflict)

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
  - `examples/lasm-alpha-full/*`
  - `docs/05-sec4-master-roadmap.md`
  - `docs/book/*`
  - `docs/book/README.md`
- Focus:
  - executable operator-facing LASM+DB demonstration
  - roadmap/book sync

## 10) Copy/Paste Prompt for Another Agent

Use this exact prompt in another editor:

---
You are working in `/Users/vladimirtrifonov/src/ai/untrusted-compiler`.

Read first:
1. `docs/codex-operator-handoff.md`
2. `docs/05-sec4-master-roadmap.md`
3. `.claude/napkin.md`

Execution mode:
- Implementation-first.
- Follow strict sequence:
  1) close strict no-stub alpha functionality checklist,
  2) implement full LASM DB client path,
  3) implement Composition Contract Analyzer (`M39-S2`),
  4) only then resume performance tuning feature work.
- Keep Cargo runs sequential.
- Run only targeted tests for touched behavior.

Git/PR contract:
- branch from `origin/dev` using `codex/<topic>` name.
- open PR to `dev`.
- enable auto-merge (`gh pr merge --auto --squash --delete-branch`).
- do not commit directly to `dev`/`main`.

For each merged chunk:
- update roadmap status + book chapter + book README,
- keep `.claude/napkin.md` updated with corrections.
---

## 11) Quick Start Commands

```bash
cd /Users/vladimirtrifonov/src/ai/untrusted-compiler
git fetch origin
git checkout dev
git pull --ff-only origin dev
git checkout -b codex/lasm-db-parity-01

# inspect state
git log --oneline -n 12
git status --short
```

## 12) Definition of Done (Per PR)

1. Real behavior implemented (no new placeholder path).
2. Focused tests green for changed behavior.
3. PR opened to `dev` and auto-merge enabled.
4. Roadmap + book docs synced.
5. No unrelated file churn.
