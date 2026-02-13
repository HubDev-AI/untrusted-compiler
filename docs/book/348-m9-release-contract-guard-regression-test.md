# M9 Slice: Release-Contract Guard Regression Test

This slice adds fixture-based regression testing for the release-contract workflow contract checker and wires it into closure/naming-lock guarantees.

## What it is

Updated:
- `scripts/test-release-contract-smoke-workflow-contract.sh`
- `scripts/test-release-contract-smoke-workflow-contract-guard.sh`
- `.github/workflows/naming-lock.yml`
- `scripts/check-milestone-closure.sh`
- `scripts/test-check-milestone-closure.sh`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/328-milestone-closure-audit-checklist.md`
- `docs/book/344-m9-release-contract-smoke-ci-closure-gate.md`

## Why it exists

Running the contract checker only against the real workflow catches drift in YAML, but not regressions in checker behavior itself (for example accidentally removing trigger checks).

A fixture-based guard test closes that gap.

## What changed

1. Made workflow checker path-overridable for fixtures
- `test-release-contract-smoke-workflow-contract.sh` now supports:
  - `--workflow <path>`

2. Added dedicated guard regression test script
- New `test-release-contract-smoke-workflow-contract-guard.sh` validates:
  - valid fixture passes,
  - missing trigger coverage fails,
  - missing publish-verifier step fails.

3. Wired guard test into naming-lock CI
- `naming-lock.yml` now runs both:
  - `scripts/test-release-contract-smoke-workflow-contract.sh`
  - `scripts/test-release-contract-smoke-workflow-contract-guard.sh`

4. Tightened closure gate `M9-F`
- `check-milestone-closure.sh` now requires naming-lock to include both contract and guard tests.
- `test-check-milestone-closure.sh` includes a negative case where guard coverage is removed.

## Validation

```bash
scripts/test-release-contract-smoke-workflow-contract.sh
scripts/test-release-contract-smoke-workflow-contract-guard.sh
scripts/test-check-milestone-closure.sh
scripts/test-roadmap-closure-gate-alignment.sh
scripts/check-milestone-closure.sh --fail-on-pending
scripts/check-naming-lock.sh
```

## Tradeoffs

- Adds another static CI contract check and fixture script to maintain.
- Gains stronger protection against regressions in checker logic, not only workflow content.
