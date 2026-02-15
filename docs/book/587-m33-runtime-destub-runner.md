# 587 M33 Runtime De-stub Runner

This chapter documents the fourth closure-gated slice in M33.

## What it is

Added:
- `scripts/run-m33-runtime-destub.sh`
- `scripts/test-run-m33-runtime-destub.sh`

Updated:
- `.github/workflows/naming-lock.yml`
- `scripts/check-milestone-closure.sh`
- `scripts/test-check-milestone-closure.sh`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/README.md`

Key behavior:
- Executes only the first selected slice from M33 runtime de-stub plan.
- Supports deterministic `--dry-run` and `--execute` modes.
- Emits execution-status and marker artifacts with closure gate `M33-D`.

## Why it exists

After deterministic planning, M33 needs a deterministic execution entrypoint to run exactly one runtime slice and produce auditable artifacts for convergence reporting.

## How it works internally

1. Validate M33 runtime de-stub plan contract (`plan[]`, `selectedTrack`, `closureGate`).
2. Select the first plan item (`plan[0]`) as the only executable slice.
3. Build domain-specific planned steps payload for artifact traceability.
4. In dry-run mode:
   - emit status JSON only,
   - do not emit execution marker.
5. In execute mode:
   - emit status JSON,
   - emit `executed-<slice>.marker`.
6. Stamp output with closure gate `M33-D`.

## Validation

Validated by:
- `scripts/test-run-m33-runtime-destub.sh`
- `scripts/test-check-milestone-closure.sh`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs and next steps

- Trade-off:
  - Runner intentionally executes only one slice per invocation to keep convergence evidence deterministic.
- Next:
  - implement `M33-S5` executed-slice convergence summary and wire `M33-E`.
