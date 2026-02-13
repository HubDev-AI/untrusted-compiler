# M9 Slice: Release-Contract-Smoke Closure Gate

This slice adds closure-audit coverage for the release-contract-smoke workflow contract.

## What it is

Updated:
- `scripts/check-milestone-closure.sh`
- `scripts/test-check-milestone-closure.sh`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/328-milestone-closure-audit-checklist.md`

## Why it exists

M9 closure already checked release gate scripts/workflows and promotion/manifest verifiers, but it did not enforce that the dedicated `release-contract-smoke` workflow keeps running all required release contract tests.

That left a drift path where M9 could appear closed while release-contract smoke wiring silently regressed.

## What changed

1. Added closure gate `M9-E`
- `check-milestone-closure.sh` now validates `.github/workflows/release-contract-smoke.yml` keeps:
  - `scripts/test-alpha-release-workflow-contract.sh`
  - `scripts/test-verify-release-promotion-inputs.sh`
  - `scripts/test-generate-release-publish-manifest.sh`
  - `scripts/test-verify-release-publish-manifest.sh`

2. Expanded closure fixture coverage
- `test-check-milestone-closure.sh` now:
  - seeds a passing release-contract-smoke workflow fixture,
  - asserts pending failure when one required publish-verifier check is removed,
  - restores the workflow fixture to valid before later checks.

3. Synced closure docs and roadmap gate table
- Roadmap strict closure table now includes `M9-E`.
- Milestone closure checklist includes the release-contract-smoke workflow contract requirements.

## Validation

```bash
scripts/test-check-milestone-closure.sh
scripts/test-roadmap-closure-gate-alignment.sh
scripts/check-milestone-closure.sh --fail-on-pending
```

## Tradeoffs

- This is static contract validation; it does not execute release workflows itself.
- It complements workflow-level runtime checks by preventing YAML drift from being treated as a closed milestone.
