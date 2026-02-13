# M9 Slice: Alpha-Release Workflow Closure Contract Gate

This slice adds closure-audit coverage for alpha-release workflow contract wiring.

## What it is

Updated:
- `scripts/check-milestone-closure.sh`
- `scripts/test-check-milestone-closure.sh`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/328-milestone-closure-audit-checklist.md`

## Why it exists

`M9-B` previously verified only that `.github/workflows/alpha-release-gate.yml` exists.

That allowed closure to pass even if required workflow steps (release gate, promotion/publish verifiers, artifact upload) drifted.

## What changed

1. Added closure gate `M9-G`
- `check-milestone-closure.sh` now verifies alpha-release workflow contract tokens:
  - `scripts/release-alpha-gate.sh`
  - `scripts/verify-release-promotion-inputs.sh`
  - `scripts/generate-release-publish-manifest.sh`
  - `scripts/verify-release-publish-manifest.sh`
  - artifact upload contract:
    - `actions/upload-artifact@v4`
    - `alpha-release-gate-artifacts`
    - `build/release-alpha-gate`

2. Expanded fixture coverage
- `test-check-milestone-closure.sh` now:
  - seeds a passing alpha-release workflow fixture,
  - asserts pending failure when artifact-upload contract is removed,
  - restores the fixture for subsequent checks.

3. Synced closure docs
- Roadmap strict closure table now includes `M9-G`.
- Milestone closure checklist now includes alpha-release workflow contract requirements.

## Validation

```bash
scripts/test-check-milestone-closure.sh
scripts/test-roadmap-closure-gate-alignment.sh
scripts/check-milestone-closure.sh --fail-on-pending
```

## Tradeoffs

- This adds static workflow contract validation to closure auditing.
- It does not execute release workflows, but it prevents closure claims when alpha workflow wiring regresses.
