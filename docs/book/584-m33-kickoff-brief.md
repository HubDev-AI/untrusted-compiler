# 584 M33 Kickoff Brief

This chapter documents the first closure-gated slice in M33.

## What it is

Added:
- `scripts/generate-m33-kickoff-brief.sh`
- `scripts/test-generate-m33-kickoff-brief.sh`

Updated:
- `.github/workflows/naming-lock.yml`
- `scripts/check-milestone-closure.sh`
- `scripts/test-check-milestone-closure.sh`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/README.md`

Key behavior:
- Generates deterministic M33 kickoff brief markdown/json from finalized M32 closure + transition packet artifacts.
- Auto-generates missing `build/m32-closure-report.json` using `build-m32-closure-report.sh`.
- Wires closure gate `M33-A` through naming-lock CI and strict closure audit checks.

## Why it exists

After M32 stabilization slices, the workflow needs a deterministic handoff artifact before opening M33 follow-up slices. The kickoff brief is the canonical start signal for the next milestone loop.

## How it works internally

1. Validate M32 transition packet JSON contract (`summary.planSelectedTrack`, `summary.planSelectedSliceId`, `summary.convergenceOverall`).
2. Ensure M32 closure report JSON exists and satisfies the expected contract (`overall`, `m32Gates[]`, packet convergence summary).
3. Compute:
   - `primaryFocus` (`stabilization` fallback if closure/convergence are not PASS),
   - `pendingGates`,
   - deterministic recommendation list.
4. Emit either JSON payload (`--format json`) or markdown brief (`--output`).
5. Enforce `M33-A` via:
   - naming-lock workflow step `scripts/test-generate-m33-kickoff-brief.sh`,
   - `scripts/check-milestone-closure.sh` gate detection and `emit_check`.

## Validation

Validated by:
- `scripts/test-generate-m33-kickoff-brief.sh`
- `scripts/test-check-milestone-closure.sh`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs and next steps

- Trade-off:
  - M33 currently wires only the kickoff gate (`M33-A`); follow-up gates are not introduced yet.
- Next:
  - implement `M33-S2` priority matrix and add `M33-B` closure-gate enforcement.
