# M9 Slice: Alpha Release Workflow Contract Test

This slice adds a CI contract test for the alpha release workflow wiring.

## What it is

Updated:
- `scripts/test-alpha-release-workflow-contract.sh`
- `.github/workflows/naming-lock.yml`

## Why it exists

`release-alpha-gate.sh` and promotion scripts can be correct while workflow YAML drifts (missing steps, renamed artifact output path, or removed verifier stages).

This slice ensures PR CI fails when the alpha release workflow no longer matches required release-promotion contract steps.

## What changed

1. Added workflow contract test script
- Validates `.github/workflows/alpha-release-gate.yml` still contains:
  - release gate command,
  - promotion-input verifier,
  - publish-manifest generation,
  - publish-manifest verifier,
  - artifact upload (`alpha-release-gate-artifacts`, `build/release-alpha-gate`).

2. Wired contract test into CI
- `naming-lock.yml` now runs:
  - `scripts/test-alpha-release-workflow-contract.sh`

## Validation

```bash
scripts/test-alpha-release-workflow-contract.sh
```

## Tradeoffs

- This is static contract validation; it does not execute release workflow logic.
- It complements runtime release script tests by protecting workflow wiring from drift.
