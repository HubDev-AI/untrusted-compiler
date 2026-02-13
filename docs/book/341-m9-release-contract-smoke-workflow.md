# M9 Slice: Release Contract Smoke Workflow

This slice adds continuous CI coverage for release/publish contract tests.

## What it is

Updated:
- `.github/workflows/release-contract-smoke.yml`

## Why it exists

Release contract tests existed but only ran when manually invoked locally.

That created a drift window where PRs could break:
- release promotion verifier assumptions,
- publish-manifest generation contract,
- publish-manifest verification contract,
without an always-on CI signal.

## What changed

Added `Release Contract Smoke` workflow (PR + `main`) that runs:
1. `scripts/test-alpha-release-workflow-contract.sh`
2. `scripts/test-verify-release-promotion-inputs.sh`
3. `scripts/test-generate-release-publish-manifest.sh`
4. `scripts/test-verify-release-publish-manifest.sh`

## Validation

```bash
scripts/test-alpha-release-workflow-contract.sh
scripts/test-verify-release-promotion-inputs.sh
scripts/test-generate-release-publish-manifest.sh
scripts/test-verify-release-publish-manifest.sh
```

## Tradeoffs

- Adds CI runtime overhead due repeated release-gate fixture generation.
- Provides deterministic PR-time detection for release-contract regressions.
