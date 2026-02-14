# 486 M19 Priority Matrix

This chapter documents M19-S2: deterministic priority ranking for M19 work tracks.

## 1) What changed

- Added matrix builder:
  - `scripts/build-m19-priority-matrix.sh`
- Added matrix contract test:
  - `scripts/test-build-m19-priority-matrix.sh`

## 2) Why it matters

After kickoff, M19 needs explicit ordering between runtime, release, and editor tracks. The matrix converts kickoff signals (`primaryFocus`, `pendingGates`, `m18Overall`) into deterministic priorities.

## 3) How it works

- Validates kickoff brief JSON contract.
- Computes per-track scores from focus + stability signals.
- Sorts into `tracks[]` with explicit `priority`.
- Emits markdown and JSON outputs.

## 4) Verification

- `scripts/test-build-m19-priority-matrix.sh`
- `scripts/check-milestone-closure.sh --fail-on-pending` now includes `M19-B`.

## 5) Tradeoff

Scoring is intentionally simple and auditable. Future M19 slices can refine weighting while preserving output contract shape.
