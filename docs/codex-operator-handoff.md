# Codex Operator Handoff (Parallel Execution Playbook)

Updated: 2026-02-18
Owner branch: `dev`
Current cut commit: `6b9ae2e`

## 1) Purpose

Use this document to onboard another Codex operator (in another editor) fast, with minimal drift.
It captures:
- where the project is now,
- what to do next,
- how to run in parallel safely,
- how to keep the same implementation-first approach.

## 2) Project Status Snapshot

### Alpha status (from roadmap)
- Runnable alpha (end-to-end): `~97-99%`
- Strict no-stub alpha: `~95-97%`

### Scope direction (locked)
- Primary focus: backend alpha completion and LASM runtime hardening.
- Defer non-essential governance loops.
- Prefer real logic/code over broad new test harnesses.

### Current milestone state
- `M39-S2A` (multi-file module system): complete.
- `M39-S2B` (LASM async backend bootstrap): active and heavily advanced.
- `M39-S3/S4` (promote dry-run/apply): already implemented baseline.

## 3) Recent Implementation Cut (what was just finished)

Latest commit window (newest first):
- `6b9ae2e` `lasm: add worker keep-alive request reuse loop`
- `0f7743a` `lasm: harden request-line parsing and http version handling`
- `266c655` `lasm: omit response body for head overload and runtime writes`
- `d5f7c7e` `lasm: return json overload envelope on queue saturation`
- `01ec9e1` `lasm: enforce json content-type for schema handlers`
- `c826e46` `lasm: align trace id across headers and error envelopes`
- `3702f41` `benchmark: include sec4-lasm in default impl sets`
- `e7e3b09` `lasm: validate benchmark user payloads in run materialization`
- `d59a414` `lasm: harden benchmark json response validation`
- `28a39ff` `lasm: materialize benchmark responses from request context`
- `cc43411` `lasm: emit deterministic json envelopes for response helpers`
- `cf2dba7` `benchmark: add first-class sec4-lasm service lane`

## 4) Files that are currently central

- Runtime/CLI LASM path:
  - `compiler/sec4-cli/src/main.rs`
  - `compiler/sec4-cli/tests/commands.rs`
- Benchmark lane:
  - `benchmark-suite/services/sec4-lasm/*`
  - `benchmark-suite/scripts/*`
- Canonical progress docs:
  - `docs/05-sec4-master-roadmap.md`
  - `docs/book/README.md`
  - `docs/book/92x-93x-*.md` (latest LASM chapters)
- Session quality log:
  - `.claude/napkin.md`

## 5) Non-negotiable Workflow Rules

1. **Implementation-first**
- Ship real runtime/compiler logic each slice.
- Add only focused tests that prove new behavior.

2. **Targeted validation only (no long test marathons by default)**
- Run the specific tests tied to the changed behavior.
- Run broad suites only when needed before PR merge.

3. **Cargo commands must be sequential**
- Never run Cargo tests/builds in parallel in this repo.

4. **Docs updated in the same slice**
- For every meaningful slice:
  - update `docs/05-sec4-master-roadmap.md` tracking line,
  - add/update one `docs/book/*` chapter,
  - update `docs/book/README.md`.

5. **Napkin updated when mistakes happen**
- Record self-corrections in `.claude/napkin.md`.

6. **Git/branch policy**
- Work from `dev` and open PRs to `dev` unless explicitly changed.
- Batch multiple related implementation slices per PR (not one tiny PR per micro-change).

## 6) Parallel Lane Strategy (low-conflict)

To reduce merge conflicts, split by file ownership where possible.

### Lane A (Runtime core behavior)
- Main files:
  - `compiler/sec4-cli/src/main.rs`
- Focus:
  - LASM connection/runtime behavior,
  - queue/timeout/keep-alive/backpressure semantics,
  - deterministic envelope/status parity.

### Lane B (Integration coverage + command UX)
- Main files:
  - `compiler/sec4-cli/tests/commands.rs`
  - optional CLI messaging in `compiler/sec4-cli/src/main.rs` only if needed.
- Focus:
  - focused command integration tests for Lane A behavior,
  - deterministic stderr/stdout checks,
  - avoid broad unrelated fixture churn.

### Lane C (Benchmark lane + service contract)
- Main files:
  - `benchmark-suite/services/sec4-lasm/*`
  - `benchmark-suite/scripts/*`
  - `benchmark-suite/README.md`
- Focus:
  - sec4-lasm benchmark comparability,
  - smoke/profile contract parity,
  - deterministic benchmark artifacts.

### Lane D (Docs sync)
- Main files:
  - `docs/05-sec4-master-roadmap.md`
  - `docs/book/*`
- Focus:
  - mirror merged implementation changes,
  - keep readiness/status accurate,
  - no speculative roadmap changes without code.

## 7) Ready-to-use Prompt for Another Codex Operator

Copy this to the other editor/operator:

---
You are working in `<repo-root>` on branch `dev`.

Read first:
1. `docs/codex-operator-handoff.md`
2. `docs/05-sec4-master-roadmap.md` (M39 section)
3. `.claude/napkin.md`

Execution contract:
- Implementation-first, focused tests only.
- Cargo commands sequential only.
- For each completed slice: update roadmap + book chapter + book README.
- Keep commits coherent and batch multiple related slices before PR.

Current cut: `6b9ae2e`.
Current priority: continue LASM backend hardening/performance behavior with deterministic HTTP/runtime contracts.

Before coding:
- show `git status --short` and `git log --oneline -n 8`.
- confirm your lane (A/B/C/D) and touched files.

After coding each slice:
- run targeted tests for that behavior,
- report exact commands + pass/fail,
- commit with a clear message,
- keep unrelated files untouched.
---

## 8) Quick Start Commands

```bash
cd <repo-root>
git checkout dev
git pull --ff-only

# Inspect current cut
git log --oneline -n 12
git status --short

# Example targeted validations
cargo test -p sec4 --test commands run_command_lasm_backend_supports_keep_alive_for_multiple_requests
cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_505_for_unsupported_http_version
bash benchmark-suite/services/sec4-lasm/smoke.sh
```

## 9) What “done” means for each slice

A slice is complete only when all are true:
- Real logic landed (not placeholders/stubs).
- Behavior proven with focused tests.
- Roadmap + book docs updated.
- Commit is clean and scoped.

## 10) Immediate Next Work Suggestions

1. LASM parser/runtime efficiency improvements (without breaking deterministic error contracts).
2. Continue queue/backpressure behavior hardening under keep-alive workloads.
3. Benchmark comparability refinements for `sec4-lasm` lane.
4. Keep docs/readiness estimate synchronized with actual merged behavior.
