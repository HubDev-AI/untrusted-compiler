# 591 M34 Kickoff Brief

This chapter documents the first closure-gated slice in M34.

## What it is

Added:
- `scripts/generate-m34-kickoff-brief.sh`
- `scripts/test-generate-m34-kickoff-brief.sh`

Updated:
- `.github/workflows/naming-lock.yml`
- `scripts/check-milestone-closure.sh`
- `scripts/test-check-milestone-closure.sh`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/README.md`

Key behavior:
- Generates deterministic M34 kickoff brief from finalized M33 closure + transition packet artifacts.
- Auto-generates missing M33 closure report JSON from `build-m33-closure-report.sh`.
- Wires closure gate `M34-A`.

## Why it exists

After M33 closure, M34 needs the same deterministic kickoff contract to start the next runtime-first stabilization loop without ambiguity.

## How it works internally

1. Validate M33 transition packet summary contract.
2. Ensure M33 closure report JSON exists and satisfies contract.
3. Derive `primaryFocus`, `pendingGates`, and recommendations.
4. Emit markdown/json kickoff brief artifacts.
5. Enforce gate `M34-A` in naming-lock and strict closure audit.

## Validation

Validated by:
- `scripts/test-generate-m34-kickoff-brief.sh`
- `scripts/test-check-milestone-closure.sh`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs and next steps

- Trade-off:
  - Kickoff output keeps deterministic minimal summary and does not inline full prior milestone artifacts.
- Next:
  - implement `M34-S2` priority matrix (`M34-B`).
