# 464 M17 Operator Handoff Checklist and Readiness Verifier

This chapter turns the M17 kickoff scope into an executable, operator-facing checklist with deterministic commands and expected outputs.

## 1) What it is

A stable handoff checklist for running the live hello-api runtime smoke profiles and validating readiness with one verifier script.

## 2) Why it exists

M16 already proves runtime behavior and CI coverage. M17-S1 packages those guarantees into a deterministic operational runbook so the same flow can be executed locally without hidden project context.

## 3) Command matrix (canonical)

1. Default runtime-smoke branch:
   - `scripts/smoke-sec4-run-hello-api.sh --artifacts-dir <tmp>/default`
2. Max-body runtime-smoke branch:
   - `scripts/smoke-sec4-run-hello-api.sh --max-body-bytes 2048 --artifacts-dir <tmp>/max-body`
3. Branch-bundle verification + index generation:
   - `scripts/check-runtime-smoke-bundle.sh --artifacts-root <tmp> --index-path <tmp>/runtime-smoke-branch-index.json`
4. Strict closure gate verification:
   - `scripts/check-milestone-closure.sh --fail-on-pending`
5. Trend-note local fallback profile:
   - `benchmark-suite/scripts/update_trend_note_from_ci.sh --prefer-local`

## 4) Expected outputs and artifacts

After the matrix passes:

- `<tmp>/default/` contains run metadata, health/users request logs, and response artifacts.
- `<tmp>/max-body/` contains the max-body runtime-smoke artifact set with matching run-flag shape metadata.
- `<tmp>/runtime-smoke-branch-index.json` exists and includes `branchOrder: ["default","max-body"]`.
- Closure gate prints `overall: PASS`.
- Trend note updater refreshes `docs/book/322-m13-first-trend-run-results-note.md` using local matrix evidence when remote fetch is unavailable.

## 5) Readiness verifier

Use the dedicated readiness script to confirm all contract files/tokens are in place:

- `scripts/check-m17-operator-handoff-readiness.sh`

Expected success output:

- `m17 operator handoff readiness check passed`

## 6) Failure modes and diagnostics

- Missing workflow command wiring:
  - `missing runtime-smoke workflow token: ...`
- Missing checklist command in chapter:
  - `missing handoff chapter token: ...`
- Non-executable required script:
  - `<label> is not executable: ...`

## 7) Verification

- `scripts/test-check-m17-operator-handoff-readiness.sh`
- `scripts/check-m17-operator-handoff-readiness.sh`
- `scripts/check-milestone-closure.sh --fail-on-pending`
