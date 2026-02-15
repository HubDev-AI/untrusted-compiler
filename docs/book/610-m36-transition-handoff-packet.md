# 610 M36 Transition Handoff Packet

This chapter documents the sixth artifact slice in M36 (`M36-S6`) for deterministic transition handoff packaging.

## What it is

Added:
- `scripts/build-m36-transition-handoff-packet.sh`
- `scripts/test-build-m36-transition-handoff-packet.sh`
- `docs/book/610-m36-transition-handoff-packet.md`

Updated:
- `docs/book/README.md`

Key behavior:
- Builds deterministic M36 handoff packet from kickoff, priority matrix, runtime plan, runtime runner, and convergence artifacts.
- Validates artifact contracts before any packet emission.
- Enforces selected track and selected slice consistency across plan, runtime execution, and convergence artifacts.
- Copies canonical packet artifacts into one output directory and emits deterministic `handoff-packet.json`.
- Stamps packet manifest with closure marker `M36-F`.
- Keeps this slice artifact-only (no closure wiring changes in this lane).

## Why it exists

M36 needs one canonical handoff artifact bundle so downstream closure reporting consumes a single deterministic manifest instead of re-reading loosely coupled execution artifacts.

## How it works internally

1. Parses CLI flags and validates `--format text|json`.
2. Requires and validates contracts for:
   - M36 kickoff brief JSON,
   - M36 priority matrix JSON,
   - M36 runtime de-stub plan JSON,
   - M36 runtime execution JSON.
3. Enforces deterministic plan/runner consistency:
   - `runtime.selectedTrack == plan.selectedTrack`
   - `runtime.selectedSlice.id == plan.plan[0].id`
4. Resolves convergence artifact:
   - uses `--convergence-json` when provided,
   - otherwise auto-generates JSON via `scripts/build-m36-executed-slice-convergence-summary.sh`.
5. Validates convergence contract and enforces:
   - `convergence.selectedTrack == plan.selectedTrack`
   - `convergence.selectedSliceId == plan.plan[0].id`
6. Copies normalized packet files:
   - `kickoff.json`
   - `priority-matrix.json`
   - `runtime-plan.json`
   - `runtime-execution.json`
   - `convergence.json`
7. Emits `handoff-packet.json` manifest with:
   - `version`, `generatedAt`,
   - summary fields (`kickoffPrimaryFocus`, `planSelectedTrack`, `planSelectedSliceId`, `runtimeStatus`, `convergenceOverall`),
   - artifacts map,
   - `closureGate: "M36-F"`.

## Inputs/outputs and constraints

Inputs (defaults):
- `build/m36-kickoff-brief.json`
- `build/m36-priority-matrix.json`
- `build/m36-runtime-destub-plan.json`
- `build/m36-runtime-destub-run/execution-status.json`
- optional `--convergence-json` override

Outputs:
- packet directory: `build/m36-transition-handoff`
- packet manifest: `build/m36-transition-handoff/handoff-packet.json`
- JSON to stdout when `--format json`
- deterministic text summary when `--format text`

Constraints:
- `jq` is required.
- Contract checks and diagnostics are stable string literals for deterministic CI assertions.
- Selected track/slice invariants must match across plan, runner, and convergence artifacts.

## Failure modes and diagnostics

Representative diagnostics:
- Missing runtime execution: `missing runtime execution json: <path>`
- Invalid kickoff contract: `invalid kickoff json contract: <path>`
- Runtime track mismatch: `runtime execution selectedTrack does not match plan track: <runtime> != <plan>`
- Runtime slice mismatch: `runtime execution selectedSlice.id does not match plan slice id: <runtime> != <plan>`
- Convergence track mismatch: `convergence selectedTrack does not match plan track: <convergence> != <plan>`
- Convergence slice mismatch: `convergence selectedSliceId does not match plan slice id: <convergence> != <plan>`

## Example usage

```bash
scripts/build-m36-transition-handoff-packet.sh \
  --kickoff-json build/m36-kickoff-brief.json \
  --matrix-json build/m36-priority-matrix.json \
  --plan-json build/m36-runtime-destub-plan.json \
  --runtime-execution-json build/m36-runtime-destub-run/execution-status.json \
  --output-dir build/m36-transition-handoff \
  --format json
```

## Validation

Validated by:
- `bash -n scripts/build-m36-transition-handoff-packet.sh`
- `bash -n scripts/test-build-m36-transition-handoff-packet.sh`
- `scripts/test-build-m36-transition-handoff-packet.sh`
- `scripts/test-build-m36-executed-slice-convergence-summary.sh` (regression)

## Trade-offs and next steps

- Trade-off:
  - Packet remains summary-first and references copied canonical artifacts instead of embedding full raw payloads.
- Next:
  - consume this handoff packet in M36 closure report generation (`M36-S7` / `M36-G`).
