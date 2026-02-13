# M13 Slice: Benchmark-Smoke Closure Contract Gate in Closure Audit

This slice brings benchmark-smoke closure wiring into strict milestone closure auditing.

## What it is

Updated:
- `scripts/check-milestone-closure.sh`
- `scripts/test-check-milestone-closure.sh`
- `scripts/test-benchmark-smoke-closure-gate.sh`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/328-milestone-closure-audit-checklist.md`
- `docs/book/333-m13-benchmark-smoke-closure-audit-gate.md`

## Why it exists

Benchmark-smoke CI already runs closure checks, but closure auditing did not explicitly validate that the workflow keeps those checks wired.

That left drift risk where closure could report PASS while workflow-level enforcement regressed.

## What changed

1. Added closure gate `M13-D`
- `check-milestone-closure.sh` now verifies `.github/workflows/benchmark-smoke.yml` keeps:
  - `scripts/test-benchmark-smoke-closure-gate.sh`
  - `scripts/test-check-milestone-closure.sh`
  - `scripts/check-milestone-closure.sh --fail-on-pending`

2. Expanded closure fixture coverage
- `test-check-milestone-closure.sh` now seeds a passing benchmark-smoke workflow fixture and asserts pending failure when closure fixture-test coverage is removed.

3. Tightened benchmark-smoke contract test
- `test-benchmark-smoke-closure-gate.sh` now also requires the closure fixture test command.

4. Synced roadmap/checklist/docs
- Roadmap strict closure table includes `M13-D`.
- Closure checklist and benchmark-smoke closure chapter now reflect the expanded contract.

## Validation

```bash
scripts/test-check-milestone-closure.sh
scripts/test-benchmark-smoke-closure-gate.sh
scripts/test-roadmap-closure-gate-alignment.sh
scripts/check-milestone-closure.sh --fail-on-pending
```

## Tradeoffs

- Adds one more static workflow contract requirement to closure auditing.
- Improves operational confidence by making closure enforcement wiring explicitly auditable.
