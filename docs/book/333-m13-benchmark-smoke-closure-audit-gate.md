# M13 Slice: Benchmark Smoke Closure Audit Gate

This slice makes closure-audit regressions fail in benchmark smoke CI, not only in local/manual runs.

## What it is

Updated:
- `.github/workflows/benchmark-smoke.yml`
- `scripts/test-benchmark-smoke-closure-gate.sh`

## Why it exists

`check-milestone-closure.sh` had fixture tests, but benchmark smoke CI did not run the strict closure audit against the repository’s committed evidence.

That left a gap where:
- fixture tests could pass,
- but real closure evidence (`compare-matrix`, trend note, workflow guardrails) could drift.

## What changed

1. Benchmark smoke CI now executes strict closure audit:
- `scripts/check-milestone-closure.sh --fail-on-pending`

2. Added workflow contract test:
- `scripts/test-benchmark-smoke-closure-gate.sh`
- Asserts benchmark-smoke workflow keeps:
  - closure gate contract test command (`scripts/test-benchmark-smoke-closure-gate.sh`)
  - closure gate guard test command (`scripts/test-benchmark-smoke-closure-gate-guard.sh`)
  - closure fixture test command (`scripts/test-check-milestone-closure.sh`)
  - strict closure-audit command (`scripts/check-milestone-closure.sh --fail-on-pending`).

## Validation

```bash
scripts/test-benchmark-smoke-closure-gate.sh
scripts/check-milestone-closure.sh --fail-on-pending
```

## Tradeoffs

- This slightly tightens CI coupling to committed closure evidence artifacts.
- It intentionally fails early when closure claims and repo evidence diverge.
