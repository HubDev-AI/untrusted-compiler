# 551 M28 Runtime Hardening Runner

This chapter documents M28-S4: the first executed M28 slice from selector output.

## 1) What changed

- Added runtime slice runner:
  - `scripts/run-m28-runtime-hardening.sh`
- Added runner contract test:
  - `scripts/test-run-m28-runtime-hardening.sh`
- Wired naming-lock + closure gate:
  - `.github/workflows/naming-lock.yml`
  - `scripts/check-milestone-closure.sh` (`M28-D`)
  - `scripts/test-check-milestone-closure.sh`

## 2) Why it matters

M28 planning artifacts require an enforced execution entrypoint. This runner turns the selected slice into executable runtime-hardening work with deterministic guardrails.

## 3) How it works

- Validates selector shape and requires:
  - `selectedTrack == runtime`,
  - recommendation ID prefix `M28-S4-runtime-`.
- Dry-run mode returns deterministic JSON/text command plans.
- Execution mode runs:
  - runtime-smoke bundle validation,
  - runtime HTTP coverage contract test,
  - strict milestone closure check.

## 4) Verification

- `scripts/test-run-m28-runtime-hardening.sh`
- `scripts/test-check-milestone-closure.sh`
- `scripts/test-roadmap-closure-gate-alignment.sh`
- `scripts/check-milestone-closure.sh --fail-on-pending`

## 5) Tradeoff

M28-S4 executes only runtime-selected slices. Release/editor execution runners remain separate future slices to keep gate/evidence progression linear.
