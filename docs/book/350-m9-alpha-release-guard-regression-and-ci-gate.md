# M9 Slice: Alpha-Release Guard Regression and CI Gate

This slice adds fixture-based regression coverage for alpha-release workflow contract checks and tracks naming-lock enforcement in closure audit.

## What it is

Updated:
- `scripts/test-alpha-release-workflow-contract.sh`
- `scripts/test-alpha-release-workflow-contract-guard.sh`
- `.github/workflows/naming-lock.yml`
- `scripts/check-milestone-closure.sh`
- `scripts/test-check-milestone-closure.sh`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/328-milestone-closure-audit-checklist.md`

## Why it exists

The alpha workflow contract checker previously ran only against the real workflow file, which does not prove the checker still fails on broken fixtures.

Also, closure auditing tracked alpha workflow contract (`M9-G`) but not whether naming-lock CI keeps both alpha contract and guard checks wired.

## What changed

1. Added workflow override to alpha contract checker
- `test-alpha-release-workflow-contract.sh` now supports:
  - `--workflow <path>`

2. Added alpha guard regression script
- New `test-alpha-release-workflow-contract-guard.sh` validates:
  - passing fixture,
  - failure when promotion verifier is missing,
  - failure when artifact upload contract is missing.

3. Wired alpha guard in naming-lock CI
- `naming-lock.yml` now runs:
  - `scripts/test-alpha-release-workflow-contract.sh`
  - `scripts/test-alpha-release-workflow-contract-guard.sh`

4. Added closure gate `M9-H`
- `check-milestone-closure.sh` now verifies naming-lock includes both alpha contract and guard tests.
- `test-check-milestone-closure.sh` includes a negative case where alpha guard coverage is removed.

## Validation

```bash
scripts/test-alpha-release-workflow-contract.sh
scripts/test-alpha-release-workflow-contract-guard.sh
scripts/test-check-milestone-closure.sh
scripts/test-roadmap-closure-gate-alignment.sh
scripts/check-milestone-closure.sh --fail-on-pending
scripts/check-naming-lock.sh
```

## Tradeoffs

- Adds one more static contract/fixture guard and one more closure gate.
- Improves confidence that alpha workflow contract checks are both behaviorally enforced and continuously wired in CI.
