# 585 M33 Priority Matrix

This chapter documents the second closure-gated slice in M33.

## What it is

Added:
- `scripts/build-m33-priority-matrix.sh`
- `scripts/test-build-m33-priority-matrix.sh`

Updated:
- `.github/workflows/naming-lock.yml`
- `scripts/check-milestone-closure.sh`
- `scripts/test-check-milestone-closure.sh`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/README.md`

Key behavior:
- Builds deterministic M33 track ranking from M33 kickoff brief JSON.
- Emits both machine-readable JSON and human-readable markdown artifacts.
- Wires closure gate `M33-B` into naming-lock CI and strict closure audit.

## Why it exists

After kickoff, M33 needs a deterministic, auditable track ordering to keep runtime-first stabilization decisions consistent across automation and manual execution.

## How it works internally

1. Validate kickoff contract (`m32Overall`, `convergenceOverall`, `primaryFocus`, `pendingGates[]`).
2. Compute base scores for `runtime`, `release`, `editor` from `primaryFocus`.
3. Apply stabilization pressure adjustments when previous milestone closure/convergence are not PASS.
4. Sort by score and assign stable priorities.
5. Persist:
   - `build/m33-priority-matrix.json`
   - `build/m33-priority-matrix.md`
6. Enforce `M33-B` via:
   - naming-lock workflow step `scripts/test-build-m33-priority-matrix.sh`,
   - `scripts/check-milestone-closure.sh` gate detection and `emit_check`.

## Validation

Validated by:
- `scripts/test-build-m33-priority-matrix.sh`
- `scripts/test-check-milestone-closure.sh`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs and next steps

- Trade-off:
  - Scoring remains intentionally heuristic and fixed-width for determinism over nuance.
- Next:
  - implement `M33-S3` runtime-first de-stub plan and gate `M33-C`.
