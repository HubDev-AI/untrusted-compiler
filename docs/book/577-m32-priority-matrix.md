# 577 M32 Priority Matrix

This chapter documents M32-S2: deterministic priority ranking for M32 runtime-first stabilization tracks.

## 1) What changed

- Added matrix builder:
  - `scripts/build-m32-priority-matrix.sh`
- Added matrix contract test:
  - `scripts/test-build-m32-priority-matrix.sh`
- Wired naming-lock + closure gate:
  - `.github/workflows/naming-lock.yml`
  - `scripts/check-milestone-closure.sh` (`M32-B`)
  - `scripts/test-check-milestone-closure.sh`

## 2) Why it matters

M32 starts from M31 closure evidence but must keep runtime de-stubbing as first-class work. The priority matrix converts kickoff signals (`primaryFocus`, `m31Overall`, `convergenceOverall`, `pendingGates`) into deterministic runtime/release/editor ordering.

## 3) How it works

- Validates M32 kickoff brief JSON contract.
- Computes per-track scores with stabilization emphasis when closure/convergence are not fully PASS.
- Sorts into `tracks[]` with explicit `priority`.
- Emits markdown and JSON outputs.

## 4) Verification

- `scripts/test-build-m32-priority-matrix.sh`
- `scripts/test-check-milestone-closure.sh`
- `scripts/test-roadmap-closure-gate-alignment.sh`
- `scripts/check-milestone-closure.sh --fail-on-pending`

## 5) Tradeoff

Scoring remains intentionally simple and auditable. Future M32 slices can refine weighting without changing output contract shape.
