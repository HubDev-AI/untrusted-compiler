# M10 Slice: Cross-Impl Guard Regression and CI Gate

This slice adds fixture-based regression tests for cross-impl workflow contract checks and tracks naming-lock enforcement in closure audit.

## What it is

Updated:
- `scripts/test-benchmark-cross-impl-workflow-contract.sh`
- `scripts/test-benchmark-cross-impl-workflow-contract-guard.sh`
- `.github/workflows/naming-lock.yml`
- `scripts/check-milestone-closure.sh`
- `scripts/test-check-milestone-closure.sh`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/328-milestone-closure-audit-checklist.md`

## Why it exists

The cross-impl workflow contract checker previously validated only the live workflow file, which does not prove the checker fails correctly on broken fixtures.

Also, closure audit tracked workflow and evidence contracts (`M10-A/B/C`) but not naming-lock CI enforcement of cross-impl contract checks.

## What changed

1. Added workflow override to cross-impl checker
- `test-benchmark-cross-impl-workflow-contract.sh` now supports:
  - `--workflow <path>`

2. Added cross-impl guard regression script
- New `test-benchmark-cross-impl-workflow-contract-guard.sh` validates:
  - passing fixture,
  - failure when strict quality flag is removed,
  - failure when artifact upload contract is removed.

3. Wired cross-impl checks into naming-lock CI
- `naming-lock.yml` now runs:
  - `scripts/test-benchmark-cross-impl-workflow-contract.sh`
  - `scripts/test-benchmark-cross-impl-workflow-contract-guard.sh`

4. Added closure gate `M10-D`
- `check-milestone-closure.sh` now verifies naming-lock includes both cross-impl contract and guard tests.
- `test-check-milestone-closure.sh` includes a negative case where cross-impl guard coverage is removed.

## Validation

```bash
scripts/test-benchmark-cross-impl-workflow-contract.sh
scripts/test-benchmark-cross-impl-workflow-contract-guard.sh
scripts/test-check-milestone-closure.sh
scripts/test-roadmap-closure-gate-alignment.sh
scripts/check-milestone-closure.sh --fail-on-pending
scripts/check-naming-lock.sh
```

## Tradeoffs

- Adds one more CI contract guard and one more closure gate.
- Improves confidence that cross-impl workflow contract checks stay behaviorally correct and continuously enforced.
