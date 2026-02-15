# 569 M31 Kickoff Brief

This chapter documents M31-S1: deterministic kickoff generation from closed M30 artifacts.

## 1) What changed

- Added kickoff brief generator:
  - `scripts/generate-m31-kickoff-brief.sh`
- Added kickoff brief contract test:
  - `scripts/test-generate-m31-kickoff-brief.sh`
- Wired naming-lock + closure gate:
  - `.github/workflows/naming-lock.yml`
  - `scripts/check-milestone-closure.sh` (`M31-A`)
  - `scripts/test-check-milestone-closure.sh`

## 2) Why it matters

M31 needs to start from closed, deterministic M30 evidence rather than manual interpretation. This kickoff brief establishes that baseline in a machine-readable and operator-readable format.

## 3) How it works

- Requires M30 transition packet contract (`summary.planSelectedTrack`, `summary.planSelectedSliceId`, `summary.convergenceOverall`).
- Uses M30 closure report JSON (auto-generates via `build-m30-closure-report.sh` if missing).
- Computes:
  - `primaryFocus` (`stabilization` fallback when closure/convergence is not PASS),
  - `pendingGates[]`,
  - deterministic recommendations.
- Emits markdown and JSON output contracts.

## 4) Verification

- `scripts/test-generate-m31-kickoff-brief.sh`
- `scripts/test-check-milestone-closure.sh`
- `scripts/test-roadmap-closure-gate-alignment.sh`
- `scripts/check-milestone-closure.sh --fail-on-pending`

## 5) Tradeoff

This slice intentionally covers only kickoff generation. Priority matrix and selector slices for M31 are deferred to keep gate/evidence progression linear and auditable.
