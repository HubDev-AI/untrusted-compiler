# 485 M19 Kickoff Brief

This chapter documents M19-S1: deterministic kickoff brief generation from finalized M18 closure evidence.

## 1) What changed

- Added kickoff brief generator:
  - `scripts/generate-m19-kickoff-brief.sh`
- Added kickoff brief contract test:
  - `scripts/test-generate-m19-kickoff-brief.sh`

The brief consumes:

- M18 closure report JSON,
- M18 transition handoff packet JSON.

## 2) Why it matters

After M18 closes, M19 should start from explicit audited state. The kickoff brief provides one deterministic artifact that summarizes closure/convergence and produces a concrete focus recommendation.

## 3) How it works

- Validates M18 packet contract.
- Accepts optional M18 closure JSON path.
- If closure JSON is missing, auto-generates it via `build-m18-closure-report.sh`.
- Computes:
  - `primaryFocus` (`stabilization` or selector track),
  - pending gate list,
  - recommendations.
- Emits markdown or JSON output.

## 4) Verification

- `scripts/test-generate-m19-kickoff-brief.sh`
- `scripts/check-milestone-closure.sh --fail-on-pending` now includes `M19-A`.

## 5) Tradeoff

The script uses M18 packet selector track as its default focus when closure is clean. If teams need different prioritization heuristics, they can extend recommendation logic in later M19 slices without changing the artifact contract.
