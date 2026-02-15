# 599 M35 Priority Matrix

This chapter documents the second closure-gated slice in M35.

## What it is

Added:
- `scripts/build-m35-priority-matrix.sh`
- `scripts/test-build-m35-priority-matrix.sh`

Updated:
- `.github/workflows/naming-lock.yml`
- `scripts/check-milestone-closure.sh`
- `scripts/test-check-milestone-closure.sh`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/README.md`

Key behavior:
- Builds deterministic M35 track ranking from M35 kickoff brief JSON.
- Emits both machine-readable JSON and human-readable markdown artifacts.
- Wires closure gate `M35-B` into naming-lock CI and strict closure audit.

## Why it exists

M35 kickoff needs the same deterministic priority contract used in recent milestones so runtime-first stabilization decisions remain auditable and reproducible.

## How it works internally

1. Validate kickoff contract fields (`m34Overall`, `convergenceOverall`, `primaryFocus`, `pendingGates[]`).
2. Compute base scores for `runtime`, `release`, and `editor` from `primaryFocus`.
3. Apply stabilization-pressure boosts to runtime when prior status is not `PASS` or gates remain pending.
4. Sort tracks deterministically by score and stable tie-breaker.
5. Emit:
   - `build/m35-priority-matrix.json`
   - `build/m35-priority-matrix.md`

## Inputs/outputs and constraints

Inputs:
- kickoff JSON (default: `build/m35-kickoff-brief.json`).

Outputs:
- JSON matrix (default: `build/m35-priority-matrix.json`).
- Markdown matrix (default: `build/m35-priority-matrix.md`).

Constraints:
- Requires `jq`.
- `primaryFocus` must be one of `runtime`, `release`, `editor`, `stabilization`.
- Contract keys must exist with expected shapes.

## Failure modes and diagnostics

- Missing kickoff file: `missing M35 kickoff brief json: <path>`.
- Invalid kickoff shape: `invalid M35 kickoff brief contract: <path>`.
- Unsupported focus: `unsupported M35 primary focus: <value>`.
- Missing dependency: `missing required command: jq`.

## Example usage

```bash
scripts/build-m35-priority-matrix.sh \
  --kickoff-json build/m35-kickoff-brief.json \
  --output-json build/m35-priority-matrix.json \
  --output-markdown build/m35-priority-matrix.md
```

## Validation

Validated by:
- `scripts/test-build-m35-priority-matrix.sh`

## Trade-offs and next steps

- Trade-off:
  - Scoring remains fixed and heuristic for deterministic behavior over adaptive weighting.
- Next:
  - implement `M35-S3` runtime-first de-stub plan and wire `M35-C`.
