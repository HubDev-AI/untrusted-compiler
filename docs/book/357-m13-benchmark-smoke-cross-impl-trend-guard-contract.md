# M13 Slice: Benchmark-Smoke Cross-Impl/Trend Guard Contract

This slice extends benchmark-smoke contract enforcement to include cross-impl and trend workflow guard checks.

## What it is

Updated:
- `.github/workflows/benchmark-smoke.yml`
- `scripts/test-benchmark-smoke-closure-gate.sh`
- `scripts/test-benchmark-smoke-closure-gate-guard.sh`
- `scripts/check-milestone-closure.sh`
- `scripts/test-check-milestone-closure.sh`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/328-milestone-closure-audit-checklist.md`
- `docs/book/333-m13-benchmark-smoke-closure-audit-gate.md`
- `docs/book/351-m13-benchmark-smoke-closure-guard-regression-test.md`

## Why it exists

Benchmark-smoke already enforced closure-audit guards, but cross-impl and trend workflow contract checks were only guaranteed through naming-lock.

Adding them to benchmark-smoke creates a second, benchmark-focused enforcement path and tightens M13 operational confidence.

## What changed

1. Benchmark-smoke workflow now runs additional contract guards
- Added to `.github/workflows/benchmark-smoke.yml`:
  - `scripts/test-benchmark-cross-impl-workflow-contract.sh`
  - `scripts/test-benchmark-cross-impl-workflow-contract-guard.sh`
  - `scripts/test-benchmark-trend-workflow-contract.sh`
  - `scripts/test-benchmark-trend-workflow-contract-guard.sh`

2. Benchmark-smoke closure contract checker tightened
- `test-benchmark-smoke-closure-gate.sh` now requires the four cross-impl/trend contract commands in addition to closure commands.

3. Guard fixtures aligned
- `test-benchmark-smoke-closure-gate-guard.sh` passing and negative fixtures now include required cross-impl/trend guard command tokens.

4. `M13-D` closure gate tightened
- `check-milestone-closure.sh` now requires benchmark-smoke to include closure + cross-impl/trend guard commands along with strict closure audit.
- `test-check-milestone-closure.sh` benchmark-smoke fixtures were updated accordingly.

## Validation

```bash
scripts/test-benchmark-smoke-closure-gate.sh
scripts/test-benchmark-smoke-closure-gate-guard.sh
scripts/test-check-milestone-closure.sh
scripts/test-roadmap-closure-gate-alignment.sh
scripts/check-milestone-closure.sh --fail-on-pending
scripts/check-naming-lock.sh
```

## Tradeoffs

- Adds extra runtime to benchmark-smoke workflow.
- Strengthens benchmark CI by requiring both closure and benchmark-workflow contract guards in one lane.
