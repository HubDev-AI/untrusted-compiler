# M9 Slice: Release-Contract-Smoke CI Closure Gate

This slice extends closure auditing with a CI guard for the release-contract-smoke workflow contract test.

## What it is

Updated:
- `scripts/check-milestone-closure.sh`
- `scripts/test-check-milestone-closure.sh`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/328-milestone-closure-audit-checklist.md`

## Why it exists

`M9-E` verifies the `release-contract-smoke` workflow keeps required release-contract test steps.

But closure could still pass if CI stopped enforcing the static workflow contract test in `naming-lock.yml`, allowing future workflow drift to slip through.

## What changed

1. Added closure gate `M9-F`
- `check-milestone-closure.sh` now validates `.github/workflows/naming-lock.yml` includes:
  - `scripts/test-release-contract-smoke-workflow-contract.sh`
  - `scripts/test-release-contract-smoke-workflow-contract-guard.sh`

2. Expanded fixture coverage
- `test-check-milestone-closure.sh` now:
  - seeds a passing naming-lock workflow fixture,
  - asserts pending failure when that guard step is removed,
  - restores the fixture before later checks.

3. Synced closure docs
- Roadmap strict closure table now includes `M9-F`.
- Milestone closure checklist includes the naming-lock CI guard requirement.

## Validation

```bash
scripts/test-check-milestone-closure.sh
scripts/test-roadmap-closure-gate-alignment.sh
scripts/check-milestone-closure.sh --fail-on-pending
```

## Tradeoffs

- This gate is static wiring validation and does not execute CI.
- It ensures closure evidence includes continuous enforcement of workflow-contract drift checks.
