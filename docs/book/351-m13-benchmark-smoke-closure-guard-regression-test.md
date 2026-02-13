# M13 Slice: Benchmark-Smoke Closure Guard Regression Test

This slice adds fixture-based regression coverage for benchmark-smoke closure contract checks and includes that guard in both CI and closure auditing.

## What it is

Updated:
- `scripts/test-benchmark-smoke-closure-gate.sh`
- `scripts/test-benchmark-smoke-closure-gate-guard.sh`
- `.github/workflows/benchmark-smoke.yml`
- `scripts/check-milestone-closure.sh`
- `scripts/test-check-milestone-closure.sh`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/328-milestone-closure-audit-checklist.md`
- `docs/book/333-m13-benchmark-smoke-closure-audit-gate.md`

## Why it exists

The benchmark-smoke closure gate checker previously validated only the live workflow file. That catches YAML drift, but not regressions in checker behavior.

Adding fixture-based guard tests closes this gap and keeps closure wiring validation self-tested.

## What changed

1. Made checker workflow path-overridable
- `test-benchmark-smoke-closure-gate.sh` now supports:
  - `--workflow <path>`

2. Added benchmark-smoke closure guard regression script
- New `test-benchmark-smoke-closure-gate-guard.sh` validates:
  - passing fixture,
  - failure when closure fixture-test command is missing,
  - failure when strict closure flag is missing.

3. Wired guard script into benchmark-smoke CI
- `benchmark-smoke.yml` now runs:
  - `scripts/test-benchmark-smoke-closure-gate.sh`
  - `scripts/test-benchmark-smoke-closure-gate-guard.sh`
  - `scripts/test-benchmark-cross-impl-workflow-contract.sh`
  - `scripts/test-benchmark-cross-impl-workflow-contract-guard.sh`
  - `scripts/test-benchmark-trend-workflow-contract.sh`
  - `scripts/test-benchmark-trend-workflow-contract-guard.sh`

4. Tightened `M13-D` closure contract
- `check-milestone-closure.sh` now requires benchmark-smoke workflow to include closure contract, guard, cross-impl/trend contract guards, fixture, and strict audit commands.
- `test-check-milestone-closure.sh` fixture coverage now includes the new guard token.

## Validation

```bash
scripts/test-benchmark-smoke-closure-gate.sh
scripts/test-benchmark-smoke-closure-gate-guard.sh
scripts/test-check-milestone-closure.sh
scripts/test-roadmap-closure-gate-alignment.sh
scripts/check-milestone-closure.sh --fail-on-pending
```

## Tradeoffs

- Adds one more script and CI test step to maintain.
- Improves confidence that closure-gate checker logic itself is guarded against silent regressions.
