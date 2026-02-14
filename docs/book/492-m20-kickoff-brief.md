# 492 M20 Kickoff Brief

This chapter documents M20-S1: deterministic kickoff generation from closed M19 artifacts.

## 1) What changed

- Added kickoff brief generator:
  - `scripts/generate-m20-kickoff-brief.sh`
- Added kickoff brief contract test:
  - `scripts/test-generate-m20-kickoff-brief.sh`
- Wired naming-lock + closure gate:
  - `.github/workflows/naming-lock.yml`
  - `scripts/check-milestone-closure.sh` (`M20-A`)
  - `scripts/test-check-milestone-closure.sh`

## 2) Why it matters

M20 needs to start from closed, deterministic M19 evidence rather than manual interpretation. This kickoff brief establishes that baseline in a machine-readable and operator-readable format.

## 3) How it works

- Requires M19 transition packet contract (`summary.selectorTrack`, `summary.selectorRecommendationId`, `summary.convergenceOverall`).
- Uses M19 closure report JSON (auto-generates via `build-m19-closure-report.sh` if missing).
- Computes:
  - `primaryFocus` (`stabilization` fallback when closure/convergence is not PASS),
  - `pendingGates[]`,
  - deterministic recommendations.
- Emits markdown and JSON output contracts.

## 4) Verification

- `scripts/test-generate-m20-kickoff-brief.sh`
- `scripts/test-check-milestone-closure.sh`
- `scripts/test-roadmap-closure-gate-alignment.sh`
- `scripts/check-milestone-closure.sh --fail-on-pending`

## 5) Tradeoff

This slice intentionally covers only kickoff generation. Priority matrix and selector slices for M20 are deferred to keep gate/evidence progression linear and auditable.
