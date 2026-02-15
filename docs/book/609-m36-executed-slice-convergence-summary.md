# 609 M36 Executed-Slice Convergence Summary

This chapter documents the fifth artifact slice in M36 (`M36-S5`) for deterministic executed-slice convergence.

## What it is

Added:
- `scripts/build-m36-executed-slice-convergence-summary.sh`
- `scripts/test-build-m36-executed-slice-convergence-summary.sh`
- `docs/book/609-m36-executed-slice-convergence-summary.md`

Updated:
- `docs/book/README.md`

Key behavior:
- Builds deterministic convergence summary from M36 runtime plan + runner artifacts.
- Enforces selected track/slice consistency between planner and runner outputs.
- Emits deterministic `executionPass`, `overall`, and `nextAction` fields.
- Emits JSON/markdown outputs with `convergenceClosureGate: M36-E`.
- Keeps this slice artifact-only (no closure wiring changes in this lane).

## Why it exists

M36 needs an auditable bridge between runtime execution artifacts and the next handoff packet slice. This summary normalizes execution outcome into one deterministic contract.

## How it works internally

1. Parses CLI flags and validates `--format markdown|json`.
2. Requires `jq` and validates both contracts:
   - M36 runtime de-stub plan (`selectedTrack`, `closureGate`, non-empty `plan[]`),
   - M36 runtime execution status (`selectedTrack`, `selectedSlice.id`, `executionStatus`, `closureGate`).
3. Enforces selected-track equality between plan and runtime execution artifacts.
4. Enforces selected-slice equality between `plan[0].id` and `selectedSlice.id`.
5. Derives `executionPass` from `executionStatus` (`PASS` and `DRY_RUN` are passing).
6. Derives deterministic `overall` + `nextAction` from `executionPass`.
7. Stamps output payload with `convergenceClosureGate: "M36-E"`.

## Inputs/outputs and constraints

Inputs (defaults):
- `build/m36-runtime-destub-plan.json`
- `build/m36-runtime-destub-run/execution-status.json`

Outputs:
- Markdown artifact file: `build/m36-executed-slice-convergence-summary.md`
- JSON to stdout when `--format json`
- Markdown file output when `--format markdown`

Constraints:
- Requires `jq`.
- Selected track and selected slice ID must exactly match across plan and runtime artifacts.
- Diagnostics are stable string literals for CI contract tests.

## Failure modes and diagnostics

Representative diagnostics:
- Missing plan: `missing M36 runtime de-stub plan json: <path>`
- Missing runtime execution: `missing M36 runtime execution json: <path>`
- Invalid plan contract: `invalid M36 runtime de-stub plan contract: <path>`
- Invalid runtime contract: `invalid M36 runtime execution contract: <path>`
- Track mismatch: `runtime execution selectedTrack does not match plan track: <runtime> != <plan>`
- Slice mismatch: `runtime execution selectedSlice.id does not match plan slice id: <runtime> != <plan>`

## Example usage

```bash
scripts/build-m36-executed-slice-convergence-summary.sh \
  --plan-json build/m36-runtime-destub-plan.json \
  --runtime-execution-json build/m36-runtime-destub-run/execution-status.json \
  --format json
```

## Validation

Validated by:
- `bash -n scripts/build-m36-executed-slice-convergence-summary.sh`
- `bash -n scripts/test-build-m36-executed-slice-convergence-summary.sh`
- `scripts/test-build-m36-executed-slice-convergence-summary.sh`
- `scripts/test-run-m36-runtime-destub.sh` (regression)

## Trade-offs and next steps

- Trade-off:
  - Convergence status is intentionally coarse (`PASS`/`PENDING`) to keep milestone gating deterministic.
- Next:
  - use this convergence artifact as a deterministic input for M36 transition handoff packet generation.
