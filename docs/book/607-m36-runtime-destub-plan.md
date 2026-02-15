# 607 M36 Runtime De-stub Plan

This chapter documents the third artifact slice in M36 (`M36-S3`) for deterministic runtime de-stub planning.

## What it is

Added:
- `scripts/plan-m36-runtime-destub.sh`
- `scripts/test-plan-m36-runtime-destub.sh`
- `docs/book/607-m36-runtime-destub-plan.md`

Updated:
- `docs/book/README.md`

Key behavior:
- Builds deterministic runtime de-stub execution order from M36 kickoff + priority matrix artifacts.
- Computes explicit `selectedTrack` and ordered plan slices with stable rationale fields (`title`, `domain`, `order`).
- Emits explicit first-slice selection metadata as `selectedSlice` + `selectedSliceRationale`.
- Forces runtime track when stabilization signals are active (non-PASS status, stabilization focus, or pending gates).
- Emits closure marker `M36-C` in the plan payload.
- Keeps this slice artifact-only (no closure wiring changes in this lane).

## Why it exists

M36 needs a reproducible plan artifact that selects the next runtime de-stub slice deterministically before execution-runner automation consumes it.

## How it works internally

1. Parses CLI flags and validates `--format text|json`.
2. Requires and validates both source contracts:
   - M36 kickoff brief (`m35Overall`, `convergenceOverall`, `primaryFocus`, `pendingGates[]`),
   - M36 priority matrix (`tracks[]` non-empty).
3. Computes `stabilizationMode` from kickoff status signals.
4. Selects track deterministically:
   - force `runtime` when stabilization mode is active,
   - otherwise use matrix top priority track.
5. Builds ordered runtime plan entries with deterministic IDs, titles, and domains.
6. Emits output JSON with explicit `closureGate: "M36-C"`, deterministic `selectedSlice`/`selectedSliceRationale`, and stable `nextAction` text.

## Inputs/outputs and constraints

Inputs (defaults):
- `build/m36-kickoff-brief.json`
- `build/m36-priority-matrix.json`

Outputs:
- JSON artifact file: `build/m36-runtime-destub-plan.json`
- JSON to stdout when `--format json`
- Deterministic text summary to stdout when `--format text`

Constraints:
- Requires `jq`.
- Track selection is limited to `runtime|release|editor`.
- Diagnostics are stable string literals for CI contract tests.

## Failure modes and diagnostics

Representative diagnostics:
- Missing kickoff: `missing M36 kickoff brief json: <path>`
- Missing matrix: `missing M36 priority matrix json: <path>`
- Invalid kickoff contract: `invalid M36 kickoff brief json contract: <path>`
- Invalid matrix contract: `invalid M36 priority matrix contract: <path>`
- Unsupported selected track: `unsupported M36 top priority track: <value>`

## Example usage

```bash
scripts/plan-m36-runtime-destub.sh \
  --kickoff-json build/m36-kickoff-brief.json \
  --matrix-json build/m36-priority-matrix.json \
  --output build/m36-runtime-destub-plan.json \
  --format json
```

## Validation

Validated by:
- `bash -n scripts/plan-m36-runtime-destub.sh`
- `bash -n scripts/test-plan-m36-runtime-destub.sh`
- `scripts/test-plan-m36-runtime-destub.sh`
- `scripts/test-build-m36-priority-matrix.sh` (regression)

## Trade-offs and next steps

- Trade-off:
  - Plan ordering is fixed heuristic logic optimized for deterministic artifacts rather than adaptive reprioritization.
- Next:
  - implement the M36 runtime de-stub runner slice that executes the first selected plan item and emits runtime status artifacts.
