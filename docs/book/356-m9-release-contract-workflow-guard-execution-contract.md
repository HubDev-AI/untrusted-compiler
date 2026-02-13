# M9 Slice: Release-Contract Workflow Guard Execution Contract

This slice tightens the release-contract-smoke workflow contract so guard-regression tests are part of the workflow itself.

## What it is

Updated:
- `.github/workflows/release-contract-smoke.yml`
- `scripts/test-release-contract-smoke-workflow-contract.sh`
- `scripts/test-release-contract-smoke-workflow-contract-guard.sh`
- `scripts/check-milestone-closure.sh`
- `scripts/test-check-milestone-closure.sh`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/328-milestone-closure-audit-checklist.md`
- `docs/book/342-m9-release-contract-smoke-workflow-contract-test.md`
- `docs/book/343-m9-release-contract-smoke-closure-gate.md`

## Why it exists

`release-contract-smoke` previously ran release/promotion/publish contract checks, but guard-regression scripts were only enforced in naming-lock.

Embedding guard tests directly in `release-contract-smoke` reduces drift risk and keeps release-contract CI self-verifying.

## What changed

1. Workflow now executes guard-regression tests
- `release-contract-smoke.yml` run block now includes:
  - `scripts/test-alpha-release-workflow-contract-guard.sh`
  - `scripts/test-release-contract-smoke-workflow-contract-guard.sh`

2. Contract checker now requires guard steps
- `test-release-contract-smoke-workflow-contract.sh` now enforces both guard tokens.

3. Guard fixture test expanded
- `test-release-contract-smoke-workflow-contract-guard.sh` fixtures were updated to include required guard steps on passing cases.
- Added negative case for missing release-contract guard step.

4. Closure gate `M9-E` tightened
- `check-milestone-closure.sh` now requires guard steps for `M9-E`.
- `test-check-milestone-closure.sh` release-contract fixtures were updated accordingly.

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

- Adds two extra test commands to `release-contract-smoke` runtime.
- In exchange, release-contract CI now validates both workflow wiring and checker behavior directly.
