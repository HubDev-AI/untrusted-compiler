# 543 M27 Next-Slice Selector

This chapter documents M27-S3: first executable slice selection from M27 kickoff + matrix artifacts.

## 1) What changed

- Added selector:
  - `scripts/select-m27-next-slice.sh`
- Added selector contract test:
  - `scripts/test-select-m27-next-slice.sh`
- Wired naming-lock + closure gate:
  - `.github/workflows/naming-lock.yml`
  - `scripts/check-milestone-closure.sh` (`M27-C`)
  - `scripts/test-check-milestone-closure.sh`

## 2) Why it matters

M27 planning artifacts become actionable only when they produce one explicit next slice. The selector provides that deterministic recommendation with stabilization fallback for carry-over risk.

## 3) How it works

- Validates kickoff + matrix contracts.
- If kickoff is not clean (`m26Overall != PASS`, `primaryFocus == stabilization`, or `pendingGates > 0`):
  - forces `M27-S4-stabilization-remediation`.
- Otherwise maps top matrix track to:
  - `M27-S4-runtime-hardening`
  - `M27-S4-release-hardening`
  - `M27-S4-editor-expansion`
- Emits `closureGate: M27-C`.

## 4) Verification

- `scripts/test-select-m27-next-slice.sh`
- `scripts/test-check-milestone-closure.sh`
- `scripts/test-roadmap-closure-gate-alignment.sh`
- `scripts/check-milestone-closure.sh --fail-on-pending`

## 5) Tradeoff

The selector chooses exactly one slice to keep execution linear and auditable. Multi-slice fan-out can be added later if needed.
