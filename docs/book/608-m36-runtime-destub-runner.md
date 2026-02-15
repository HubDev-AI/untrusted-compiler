# 608 M36 Runtime De-stub Runner

This chapter documents the fourth artifact slice in M36 (`M36-S4`) for deterministic runtime de-stub execution.

## What it is

Added:
- `scripts/run-m36-runtime-destub.sh`
- `scripts/test-run-m36-runtime-destub.sh`
- `docs/book/608-m36-runtime-destub-runner.md`

Updated:
- `docs/book/README.md`

Key behavior:
- Executes only the first selected slice from the M36 runtime de-stub plan.
- Supports deterministic `--dry-run` and `--execute` modes.
- Emits `execution-status.json` with selected slice/domain metadata and closure marker `M36-D`.
- Emits `executed-<slice>.marker` only in `--execute` mode.
- Keeps this slice artifact-only (no closure wiring changes in this lane).

## Why it exists

After deterministic M36 planning (`M36-C`), the milestone needs a deterministic runner artifact that can execute exactly one selected runtime slice and emit auditable status output for downstream convergence work.

## How it works internally

1. Parses CLI flags and validates `--format text|json`.
2. Requires `jq` and validates the M36 runtime plan contract (`plan[]`, `selectedTrack`, `closureGate`).
3. Selects only the first plan item (`plan[0]`) as the executable slice.
4. Resolves deterministic planned-step templates from selected domain (`db`, `net`, `validators`, `fs`, `secrets`).
5. Emits status payload in both modes with stable fields:
   - `selectedTrack`
   - `selectedSlice.id`
   - `selectedSlice.domain`
   - `runMode`
   - `executionStatus`
   - `closureGate: "M36-D"`
6. Writes execution marker only when `--execute` is set.

## Inputs/outputs and constraints

Inputs (defaults):
- `build/m36-runtime-destub-plan.json`

Outputs:
- JSON status artifact: `build/m36-runtime-destub-run/execution-status.json`
- Marker artifact in execute mode: `build/m36-runtime-destub-run/executed-<slice>.marker`
- JSON to stdout when `--format json`
- Deterministic text summary when `--format text`

Constraints:
- Requires `jq`.
- Supports domains: `db`, `net`, `validators`, `fs`, `secrets`.
- Diagnostics remain stable string literals for contract tests.

## Failure modes and diagnostics

Representative diagnostics:
- Missing plan: `missing M36 runtime de-stub plan json: <path>`
- Invalid plan contract: `invalid M36 runtime de-stub plan contract: <path>`
- Unsupported domain: `unsupported M36 runtime de-stub domain: <value>`
- Unknown CLI format: `unknown format: <value>`

## Example usage

```bash
scripts/run-m36-runtime-destub.sh \
  --plan-json build/m36-runtime-destub-plan.json \
  --output-dir build/m36-runtime-destub-run \
  --dry-run \
  --format json
```

## Validation

Validated by:
- `bash -n scripts/run-m36-runtime-destub.sh`
- `bash -n scripts/test-run-m36-runtime-destub.sh`
- `scripts/test-run-m36-runtime-destub.sh`
- `scripts/test-plan-m36-runtime-destub.sh` (regression)

## Trade-offs and next steps

- Trade-off:
  - The runner executes only one slice per invocation, favoring deterministic evidence over throughput.
- Next:
  - use emitted runtime status artifact as the deterministic input for M36 convergence summary generation.
