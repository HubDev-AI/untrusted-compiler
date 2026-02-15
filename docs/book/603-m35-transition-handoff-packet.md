# 603 M35 Transition Handoff Packet

This chapter documents the sixth closure-gated slice in M35.

## What

Added:
- `scripts/build-m35-transition-handoff-packet.sh`
- `scripts/test-build-m35-transition-handoff-packet.sh`
- `docs/book/603-m35-transition-handoff-packet.md`

Updated:
- `docs/book/README.md`

Key behavior:
- Validates kickoff/matrix/plan/runtime/convergence artifact contracts for M35.
- Enforces selected track/slice consistency across runtime execution and convergence artifacts.
- Copies normalized packet artifacts and emits deterministic `handoff-packet.json`.
- Stamps the packet with closure gate `M35-F`.

## Why

M35 requires one deterministic runtime-artifact packet that can be handed to closure reporting without re-evaluating planner and runner state from multiple files.

## How

1. Parse CLI inputs with deterministic defaults for M35 artifacts.
2. Validate required artifact contracts (`kickoff`, `priority matrix`, `runtime plan`, `runtime execution`).
3. Enforce consistency invariants:
   - `runtime.selectedTrack == plan.selectedTrack`
   - `runtime.selectedSlice.id == plan.plan[0].id`
4. Resolve convergence artifact:
   - use `--convergence-json` when provided,
   - otherwise auto-generate with `scripts/build-m35-executed-slice-convergence-summary.sh`.
5. Validate convergence contract and enforce:
   - `convergence.selectedTrack == plan.selectedTrack`
   - `convergence.selectedSliceId == plan.plan[0].id`
6. Copy packet files into output directory:
   - `kickoff.json`
   - `priority-matrix.json`
   - `runtime-plan.json`
   - `runtime-execution.json`
   - `convergence.json`
7. Emit `handoff-packet.json` manifest with:
   - `version`, `generatedAt`,
   - `summary` (`kickoffPrimaryFocus`, `planSelectedTrack`, `planSelectedSliceId`, `runtimeStatus`, `convergenceOverall`),
   - `artifacts` map,
   - `closureGate: "M35-F"`.

## Validation

Validated by:
- `bash -n scripts/build-m35-transition-handoff-packet.sh`
- `bash -n scripts/test-build-m35-transition-handoff-packet.sh`
- `scripts/test-build-m35-transition-handoff-packet.sh`
- `scripts/test-build-m35-executed-slice-convergence-summary.sh`

## Local Iteration Note

- Use `FAST=1 scripts/test-check-milestone-closure.sh` for local iteration when you need quick deterministic closure signal focused on latest M35 gates.
- Before merge, run the full `scripts/test-check-milestone-closure.sh` (without `FAST`) to preserve exhaustive coverage.

## Trade-offs

- The packet is summary-first; it references canonical copied artifacts instead of inlining all source payloads.
- Convergence auto-generation simplifies usage but introduces an implicit dependency on the convergence-summary builder script.

## Next

- Implement M35-S7 closure report and wire final closure gate `M35-G`.
