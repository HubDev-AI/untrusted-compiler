# 606 M36 Priority Matrix

This chapter documents the second artifact slice in M36 (`M36-S2`) for deterministic track prioritization.

## What it is

Added:
- `scripts/build-m36-priority-matrix.sh`
- `scripts/test-build-m36-priority-matrix.sh`
- `docs/book/606-m36-priority-matrix.md`

Updated:
- `docs/book/README.md`

Key behavior:
- Builds deterministic M36 track ranking from M36 kickoff brief JSON.
- Validates kickoff contract shape before any scoring is applied.
- Emits both machine-readable JSON and human-readable markdown artifacts.
- Includes closure metadata marker `M36-B` in generated matrix output.
- Keeps this slice scoped to artifact generation only (no closure wiring changes).

## Why it exists

M36 needs a stable, auditable priority matrix artifact immediately after kickoff so runtime/release/editor sequencing can be reproduced in CI and handoff flows.

## How it works internally

1. Parses CLI flags and validates `--format text|json`.
2. Requires kickoff input (default `build/m36-kickoff-brief.json`) and validates mandatory contract fields.
3. Enforces deterministic status/enum checks (`primaryFocus`, `selectedTrack`, status fields, gate marker).
4. Computes explicit numeric scores for `runtime`, `release`, and `editor` using:
   - base focus weighting,
   - selected-track boost,
   - stabilization pressure from overall status, convergence status, runtime status, and pending gate count.
5. Sorts tracks by descending score with stable tie-break (`track`) and assigns priority indices.
6. Emits:
   - `build/m36-priority-matrix.json`
   - `build/m36-priority-matrix.md`

## Inputs/outputs and constraints

Inputs (default):
- `build/m36-kickoff-brief.json`

Outputs:
- JSON artifact: `build/m36-priority-matrix.json`
- Markdown artifact: `build/m36-priority-matrix.md`

Constraints:
- Requires `jq`.
- Kickoff contract must include required fields and consistent pending gate count.
- Kickoff gate marker must be `M36-A`.
- Closure marker is fixed in output as `M36-B`.

## Failure modes and diagnostics

Representative diagnostics:
- Missing kickoff file: `missing M36 kickoff brief json: <path>`
- Invalid kickoff contract: `invalid M36 kickoff brief contract: <path>`
- Unexpected kickoff gate: `unexpected M36 kickoff gate marker: <value>`
- Unsupported status/focus values:
  - `unsupported M36 primary focus: <value>`
  - `unsupported M36 selected track: <value>`
  - `unsupported M36 runtimeStatus: <value>`

## Example usage

```bash
scripts/build-m36-priority-matrix.sh \
  --kickoff-json build/m36-kickoff-brief.json \
  --output-json build/m36-priority-matrix.json \
  --output-markdown build/m36-priority-matrix.md \
  --format json
```

## Validation

Validated by:
- `bash -n scripts/build-m36-priority-matrix.sh`
- `bash -n scripts/test-build-m36-priority-matrix.sh`
- `scripts/test-build-m36-priority-matrix.sh`
- `scripts/test-build-m36-kickoff-brief.sh` (regression)

## Trade-offs and next steps

- Trade-off:
  - Scoring is heuristic and intentionally fixed for deterministic CI diffs, not adaptive optimization.
- Next:
  - implement the M36 next-slice selector artifact using this matrix as ranked input while keeping closure wiring in a separate slice.
