# 542 M27 Priority Matrix

This chapter documents M27-S2: deterministic priority ranking for M27 work tracks.

## 1) What changed

- Added matrix builder:
  - `scripts/build-m27-priority-matrix.sh`
- Added matrix contract test:
  - `scripts/test-build-m27-priority-matrix.sh`
- Wired naming-lock + closure gate:
  - `.github/workflows/naming-lock.yml`
  - `scripts/check-milestone-closure.sh` (`M27-B`)
  - `scripts/test-check-milestone-closure.sh`

## 2) Why it matters

After kickoff, M27 needs explicit ordering between runtime, release, and editor tracks. The matrix converts kickoff signals (`primaryFocus`, `pendingGates`, `m26Overall`) into deterministic priorities.

## 3) How it works

- Validates M27 kickoff brief JSON contract.
- Computes per-track scores from focus + stabilization signals.
- Sorts into `tracks[]` with explicit `priority`.
- Emits markdown and JSON outputs.

## 4) Verification

- `scripts/test-build-m27-priority-matrix.sh`
- `scripts/test-check-milestone-closure.sh`
- `scripts/test-roadmap-closure-gate-alignment.sh`
- `scripts/check-milestone-closure.sh --fail-on-pending`

## 5) Tradeoff

Scoring is intentionally simple and auditable. Future M27 slices can refine weighting while preserving output contract shape.
