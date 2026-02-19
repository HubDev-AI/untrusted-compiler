# Codex Operator Handoff (Multi-Agent Fast Track)

Updated: 2026-02-19  
Primary branch: `dev`  
Current baseline commit: `d4cf6ad`

## 1) Purpose

This document is the execution contract for parallel Codex operators.
Use it to keep speed high without losing architecture direction.

## 2) Current Reality (Do Not Drift)

1. Multi-file modules are done (`M39-S2A` complete).
2. LASM async runtime is heavily implemented (`M39-S2B` advanced and benchmarked).
3. Main remaining technical gap for current priority is LASM DB parity.
4. Composition Contract Analyzer (`M39-S2`) is deferred and not a current blocker.

## 3) Backlog Priority (Immediate)

### P0: LASM DB parity (implementation-first)

1. Replace LASM DB schema-name materialization bridge with real LASM DB intrinsic execution path.
2. Implement deterministic DB handle/transaction semantics in LASM runtime path.
3. Keep persistent storage adapter (`records.log`) as v1 backend, but execute through intrinsic flow, not response-schema switches.
4. Align deterministic error envelopes with existing runtime contracts.
5. Extend the LASM full example to demonstrate real DB intrinsic path end-to-end.

### P1: Hardening after P0

1. Load/capacity verification on LASM DB paths.
2. Perf cleanup in hot paths found during profiling.
3. Additional parity fixes vs C runtime only where behavior differs.

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
