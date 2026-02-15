# 586 M33 Runtime De-stub Plan

This chapter documents the third closure-gated slice in M33.

## What it is

Added:
- `scripts/plan-m33-runtime-destub.sh`
- `scripts/test-plan-m33-runtime-destub.sh`

Updated:
- `.github/workflows/naming-lock.yml`
- `scripts/check-milestone-closure.sh`
- `scripts/test-check-milestone-closure.sh`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/README.md`

Key behavior:
- Builds deterministic runtime-first de-stub execution order from M33 kickoff + priority matrix artifacts.
- Forces runtime track when stabilization signals are active (pending gates / non-PASS convergence).
- Exposes explicit closure metadata with gate `M33-C`.

## Why it exists

M33 needs a reproducible plan that determines which runtime domains are de-stubbed first, so execution runner slices remain deterministic and auditable.

## How it works internally

1. Validate kickoff and priority-matrix contracts.
2. Compute stabilization mode from:
   - `m32Overall`,
   - `convergenceOverall`,
   - `primaryFocus`,
   - `pendingGates` length.
3. Choose `selectedTrack`:
   - always `runtime` in stabilization mode,
   - otherwise top matrix track.
4. Emit ordered plan entries (`id`, `title`, `domain`, `order`) and `nextAction`.
5. Stamp closure gate `M33-C` in output payload.
6. Enforce gate via naming-lock + closure checker wiring.

## Validation

Validated by:
- `scripts/test-plan-m33-runtime-destub.sh`
- `scripts/test-check-milestone-closure.sh`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs and next steps

- Trade-off:
  - Plan ordering is fixed heuristic logic, optimized for determinism over dynamic adaptation.
- Next:
  - implement `M33-S4` runtime de-stub execution runner and wire `M33-D`.
