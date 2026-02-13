# M9 Slice: Release-Contract-Smoke Workflow Contract Test

This slice adds static CI contract validation for the `release-contract-smoke` workflow.

## What it is

Updated:
- `scripts/test-release-contract-smoke-workflow-contract.sh`
- `.github/workflows/naming-lock.yml`

## Why it exists

A workflow can drift after being introduced (missing script calls, renamed steps), which weakens the intended release-contract safety net.

This slice keeps required test-step wiring explicit and CI-enforced.

## What changed

1. Added workflow contract test script
- Validates `.github/workflows/release-contract-smoke.yml` includes:
  - `scripts/test-alpha-release-workflow-contract.sh`
  - `scripts/test-verify-release-promotion-inputs.sh`
  - `scripts/test-generate-release-publish-manifest.sh`
  - `scripts/test-verify-release-publish-manifest.sh`

2. Wired contract test into naming-lock CI
- `naming-lock.yml` now runs:
  - `scripts/test-release-contract-smoke-workflow-contract.sh`

## Validation

```bash
scripts/test-release-contract-smoke-workflow-contract.sh
```

## Tradeoffs

- Static contract check does not execute release-contract tests itself.
- Complements runtime `release-contract-smoke` workflow by guarding against YAML drift.
