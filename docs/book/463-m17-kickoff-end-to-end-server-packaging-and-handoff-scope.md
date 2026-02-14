# 463 M17 Kickoff: End-to-End Server Packaging and Handoff Scope

This chapter captures the kickoff scope for post-M16 work focused on operator-ready end-to-end server packaging and handoff.

## 1) What it is

M17 starts from a working runtime (`sec4 run`) and closure-gated runtime-smoke flow, and turns it into a deterministic operator handoff track.

## 2) Why it exists

M16 established strong runtime and CI contracts. M17 ensures this is easy to execute by operators with a clear command matrix and expected artifacts.

## 3) Scope at kickoff

Included:

- canonical bootstrap command profiles
- runtime-smoke branch/bundle verification matrix
- operator handoff checklist aligned with closure gates

Deferred:

- non-essential runtime feature expansion
- post-alpha release channel automation beyond existing M9 gates

## 4) Initial command matrix (kickoff baseline)

1. Default runtime-smoke:
   - `scripts/smoke-sec4-run-hello-api.sh --artifacts-dir <tmp>/default`
2. Max-body runtime-smoke:
   - `scripts/smoke-sec4-run-hello-api.sh --max-body-bytes 2048 --artifacts-dir <tmp>/max-body`
3. Bundle validation/index:
   - `scripts/check-runtime-smoke-bundle.sh --artifacts-root <tmp> --index-path <tmp>/runtime-smoke-branch-index.json`
4. Closure readiness:
   - `scripts/check-milestone-closure.sh --fail-on-pending`
5. Trend-note local fallback:
   - `benchmark-suite/scripts/update_trend_note_from_ci.sh --prefer-local`

## 5) Expected artifacts

- runtime-smoke branch directories:
  - `<tmp>/default/*`
  - `<tmp>/max-body/*`
- aggregated branch index:
  - `<tmp>/runtime-smoke-branch-index.json`
- trend note update target:
  - `docs/book/322-m13-first-trend-run-results-note.md`

## 6) Next implementation slice (M17-S1)

Implement a dedicated operator handoff checklist chapter with:

- explicit pass/fail expectations for each command,
- artifact inspection checklist,
- common failure diagnostics and recovery steps.

## Verification

- `scripts/check-milestone-closure.sh --fail-on-pending`
- `scripts/test-roadmap-closure-gate-alignment.sh`
