# M10 Slice: Cross-Impl Workflow Closure Contract Gate

This slice adds closure-audit coverage for the cross-implementation evidence workflow wiring.

## What it is

Updated:
- `scripts/check-milestone-closure.sh`
- `scripts/test-check-milestone-closure.sh`

## Why it exists

M10 closure previously validated committed matrix evidence (`M10-A`, `M10-B`) but not the workflow wiring that produces future cross-impl evidence.

That allowed a drift risk where current evidence passes while the workflow contract regresses.

## What changed

1. Added closure gate `M10-C`
- Validates `.github/workflows/benchmark-cross-impl-evidence.yml` contract:
  - scoped run command is present,
  - scoped impl set `sec4,node,go,rust` is present,
  - scoped endpoint set `ping,decode` is present,
  - strict quality gate (`--fail-on-warning`) is present,
  - artifact upload contract (`benchmark-cross-impl-evidence`, `benchmark-suite/results`) is present.

2. Expanded fixture coverage
- `test-check-milestone-closure.sh` now:
  - includes passing cross-impl workflow fixture,
  - asserts pending failure when strict quality flag is removed.

## Validation

```bash
scripts/test-check-milestone-closure.sh
scripts/check-milestone-closure.sh --fail-on-pending
```

## Tradeoffs

- Static contract validation catches workflow drift early, but does not execute cross-impl benchmark runs.
- Complements runtime benchmark-smoke/workflow tests by keeping closure contracts deterministic.
