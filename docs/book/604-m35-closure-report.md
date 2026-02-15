# 604 M35 Closure Report

This chapter documents the seventh artifact slice in M35 for runtime closure reporting.

## What it is

Added:
- `scripts/build-m35-closure-report.sh`
- `scripts/test-build-m35-closure-report.sh`
- `docs/book/604-m35-closure-report.md`

Updated:
- `docs/book/README.md`

Key behavior:
- Builds deterministic M35 closure report JSON/markdown from strict closure gates plus M35 runtime artifacts.
- Validates consistency across kickoff, matrix, runtime plan, runtime execution, convergence summary, and transition packet.
- Auto-generates missing M35 transition handoff packet via `scripts/build-m35-transition-handoff-packet.sh`.
- Stamps closure artifact marker `M35-G`.

## Why it exists

M35 needs a single closure artifact that summarizes runtime execution convergence and strict gate state before moving to the next milestone kickoff.

## How it works internally

1. Loads strict closure gate snapshot from `check-milestone-closure.sh --format json` (or explicit `--closure-json`).
2. Validates contracts for:
   - kickoff brief,
   - priority matrix,
   - runtime de-stub plan,
   - runtime execution status,
   - executed-slice convergence summary,
   - transition handoff packet.
3. If transition packet is missing, calls `scripts/build-m35-transition-handoff-packet.sh` to generate it.
4. Enforces cross-artifact consistency:
   - selected track/slice alignment (plan/runtime/convergence/packet),
   - runtime/convergence summary alignment,
   - packet artifact basename alignment,
   - expected gate markers (`M35-E`, `M35-F`).
5. Evaluates required closure gates `M35-A..M35-F`.
6. Computes deterministic closure outcome and emits closure marker `M35-G`.

## Inputs/outputs and constraints

Inputs (defaults):
- `build/m35-kickoff-brief.json`
- `build/m35-priority-matrix.json`
- `build/m35-runtime-destub-plan.json`
- `build/m35-runtime-destub-run/execution-status.json`
- `build/m35-transition-handoff/convergence.json`
- `build/m35-transition-handoff/handoff-packet.json`

Outputs:
- Markdown: `build/m35-closure-report.md` (default)
- JSON: stdout when `--format json`

Constraints:
- `jq` is required.
- Closure report stays deterministic and string-literal stable for audit automation.
- This slice intentionally avoids closure-gate wiring updates in shared workflow/audit docs files.

## Failure modes and diagnostics

Representative diagnostics:
- Missing kickoff: `missing M35 kickoff brief json: <path>`
- Missing convergence: `missing M35 executed-slice convergence summary json: <path>`
- Invalid transition packet: `invalid M35 transition packet json contract: <path>`
- Track/slice mismatch: `transition packet planSelectedSliceId does not match runtime plan slice id: ...`

## Example usage

```bash
scripts/build-m35-closure-report.sh \
  --format json \
  --output build/m35-closure-report.md
```

## Trade-offs and next steps

- Trade-off:
  - Closure report enforces strict structural/consistency checks, which increases up-front validation but keeps milestone state auditable.
- Next:
  - integrate `M35-G` closure gate wiring in the shared naming-lock/closure audit lane.
