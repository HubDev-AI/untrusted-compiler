# M9 Slice: Release-Contract Workflow Trigger Contract

This slice hardens the `release-contract-smoke` workflow contract test with trigger requirements.

## What it is

Updated:
- `scripts/test-release-contract-smoke-workflow-contract.sh`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/342-m9-release-contract-smoke-workflow-contract-test.md`

## Why it exists

The workflow contract test already pinned required release/promotion/publish test steps, but it did not enforce trigger posture.

That allowed drift where steps remained present while workflow execution scope regressed (for example, no PR coverage or no `main` push coverage).

## What changed

1. Added trigger contract checks
- `test-release-contract-smoke-workflow-contract.sh` now requires:
  - `pull_request:` trigger,
  - `push:` trigger,
  - `branches:` block,
  - `- main` branch target.

2. Kept existing step contract checks
- The script continues to enforce all required release-contract test steps in the workflow body.

3. Synced roadmap/book notes
- Closure hardening notes now include trigger coverage.
- Chapter `342` now documents the trigger contract in the test scope.

## Validation

```bash
scripts/test-release-contract-smoke-workflow-contract.sh
scripts/check-naming-lock.sh
```

## Tradeoffs

- This remains static contract validation over workflow YAML.
- It complements runtime execution workflows by guarding both trigger and step wiring drift.
