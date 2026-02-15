# 605 M36 Kickoff Brief

This chapter documents the first artifact slice in M36 for deterministic kickoff intent generation.

## What it is

Added:
- `scripts/build-m36-kickoff-brief.sh`
- `scripts/test-build-m36-kickoff-brief.sh`
- `docs/book/605-m36-kickoff-brief.md`

Updated:
- `docs/book/README.md`

Key behavior:
- Builds deterministic M36 kickoff brief JSON/text from finalized M35 closure report and transition handoff packet.
- Validates closure/packet contracts and enforces cross-artifact summary consistency.
- Emits explicit M36 kickoff intent (`m36KickoffIntent`) and deterministic gate marker (`M36-A`) in the brief payload.
- Keeps this kickoff slice scoped to artifact generation only (no closure wiring changes in this slice).

## Why it exists

M36 needs a deterministic kickoff artifact that turns M35 closure evidence into an explicit next-milestone intent before broader runtime/release/editor follow-up work starts.

## How it works internally

1. Parses CLI flags and validates `--format text|json`.
2. Requires and validates:
   - M35 closure report JSON contract,
   - M35 transition handoff packet JSON contract.
3. Enforces deterministic cross-artifact summary alignment for:
   - kickoff primary focus,
   - selected track,
   - selected slice id,
   - runtime status,
   - convergence overall.
4. Verifies gate markers are canonical:
   - closure report gate `M35-G`,
   - transition packet gate `M35-F`.
5. Computes deterministic M36 kickoff posture:
   - `primaryFocus` follows selected track only when M35 is fully PASS with no pending gates,
   - otherwise falls back to `stabilization`.
6. Emits kickoff brief JSON with `kickoffGate: "M36-A"` and explicit `m36KickoffIntent`, then prints either JSON or text summary output.

## Inputs/outputs and constraints

Inputs (defaults):
- `build/m35-closure-report.json`
- `build/m35-transition-handoff/handoff-packet.json`

Outputs:
- JSON artifact file: `build/m36-kickoff-brief.json` (default `--output`)
- JSON to stdout when `--format json`
- Deterministic text summary to stdout when `--format text`

Constraints:
- `jq` is required.
- Contracts and diagnostics stay string-literal stable for automation.
- Scope is intentionally limited to kickoff artifact generation (no closure gate wiring files touched here).

## Failure modes and diagnostics

Representative diagnostics:
- Missing closure report: `missing M35 closure report json: <path>`
- Missing handoff packet: `missing M35 transition packet json: <path>`
- Invalid closure contract: `invalid M35 closure report json contract: <path>`
- Invalid packet contract: `invalid M35 transition packet json contract: <path>`
- Cross-artifact mismatch:
  - `M35 closure report packetSummary.planSelectedSliceId does not match transition packet summary.planSelectedSliceId: ...`
- Unexpected gate marker:
  - `unexpected M35 closure report gate marker: ...`
  - `unexpected M35 transition packet gate marker: ...`

## Example usage

```bash
scripts/build-m36-kickoff-brief.sh \
  --m35-closure-json build/m35-closure-report.json \
  --m35-packet-json build/m35-transition-handoff/handoff-packet.json \
  --output build/m36-kickoff-brief.json \
  --format json
```

## Trade-offs and next steps

- Trade-off:
  - The brief enforces strict consistency and canonical gate markers up front, which may fail fast when upstream artifacts drift but keeps milestone transitions auditable.
- Next:
  - wire `M36-A` into shared closure/audit workflows in the dedicated wiring slice.
