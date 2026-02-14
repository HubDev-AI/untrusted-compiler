# 487 M19 Next-Slice Selector

This chapter documents M19-S3: first executable slice selection from M19 kickoff + matrix artifacts.

## 1) What changed

- Added selector:
  - `scripts/select-m19-next-slice.sh`
- Added selector contract test:
  - `scripts/test-select-m19-next-slice.sh`

## 2) Why it matters

M19 planning artifacts become actionable only when they produce one explicit next slice. The selector provides that deterministic recommendation with fallback to stabilization when carry-over risk exists.

## 3) How it works

- Validates kickoff + matrix contracts.
- If kickoff is not clean (`m18Overall != PASS`, `primaryFocus == stabilization`, or `pendingGates > 0`):
  - forces `M19-S2-stabilization-remediation`.
- Otherwise maps top matrix track to:
  - `M19-S2-runtime-hardening`
  - `M19-S2-release-hardening`
  - `M19-S2-editor-expansion`
- Emits `closureGate: M19-C`.

## 4) Verification

- `scripts/test-select-m19-next-slice.sh`
- `scripts/check-milestone-closure.sh --fail-on-pending` now includes `M19-C`.

## 5) Tradeoff

The selector currently chooses exactly one slice and one closure gate to keep execution linear and auditable. Multi-slice fan-out can be added later if needed.
