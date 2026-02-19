# Codex Operator Handoff (Multi-Agent Fast Track)

Updated: 2026-02-19  
Primary branch: `dev`  
Current baseline commit: `4de2d06`

## 1) Purpose

This document is the execution contract for parallel Codex operators.
Use it to keep speed high without losing architecture direction.

## 2) Current Reality (Do Not Drift)

1. Multi-file modules are done (`M39-S2A` complete).
2. LASM async runtime is heavily implemented (`M39-S2B` advanced and benchmarked).
3. LASM DB parity for active intrinsics is done on file-backed adapter v1 (`records.log`).
4. Built-in LASM horizontal front-layer automation is now in-progress/usable (`sec4 run --instances ...` with autoscale flags).
5. Composition Contract Analyzer (`M39-S2`) remains deferred and not a current blocker.

## 3) Backlog Priority (Immediate)

### P0: LASM scale hardening to higher RPS ceilings (implementation-first)

1. Optimize front proxy + worker dispatch hot path (current bottleneck before 1M req/s goal).
2. Reduce per-connection overhead in cluster mode (threading/copy path) and re-benchmark.
3. Keep deterministic overload/error behavior while tuning performance.
4. Produce repeatable throughput+latency+RSS evidence from cluster mode runs.
5. Continue tuning toward higher ceilings before claiming production-scale target.

### P1: DB adapter progression after P0

1. Keep file adapter (`records.log`) as alpha v1.
2. Add SQLite adapter behind same DB intrinsic surface.
3. Keep external DB adapters post-alpha.

### P2: Fixed order after DB integration (must follow)

1. Keep LASM as default server backend for `sec4 run` (already active; C remains explicit fallback).
2. Complete LASM stability/load hardening on that default path (throughput/latency/memory regressions tracked).
3. Move DB adapter progression behind same intrinsic surface (SQLite next).
4. Finalize one canonical LASM+DB operator flow (`init/check/build/run/load-test`) with reproducible docs.
5. Only then move additional capacity to WASM/browser track.

## 4) DB Status (Explicit)

Current LASM DB is **not** a full DB client yet.

It currently works as a compatibility bridge in CLI runtime:

- Dynamic state + persistence:
  - `compiler/sec4-cli/src/main.rs`
- Stores records in `records.log` under `--db-base` / `SEC4_RT_LASM_DB_BASE`.
- DB routes are currently resolved through response-schema branches (`DbExecResponse`, `DbExecTxResponse`, `DbQueryOneResponse`, `DbListRecordsResponse`) instead of full intrinsic dispatch.

C runtime DB path is already real in `runtime/c/sec4_runtime.c`; LASM needs parity-style intrinsic execution.

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
You are working in `/Users/vladimirtrifonov/src/ai/AILang`.

Read first:
1. `docs/codex-operator-handoff.md`
2. `docs/05-sec4-master-roadmap.md`
3. `.claude/napkin.md`

Execution mode:
- Implementation-first.
- Focus now: LASM DB parity (real intrinsic path), not composition analyzer.
- After DB parity: follow P2 order exactly (LASM default -> load hardening -> SQLite adapter -> alpha usability -> WASM).
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
cd /Users/vladimirtrifonov/src/ai/AILang
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
