# M13 Slice: Trend Guard Regression and CI Gate

This slice adds fixture-based regression coverage for benchmark-trend workflow contract checks and tracks naming-lock enforcement in closure auditing.

## What it is

Updated:
- `scripts/test-benchmark-trend-workflow-contract.sh`
- `scripts/test-benchmark-trend-workflow-contract-guard.sh`
- `.github/workflows/naming-lock.yml`
- `scripts/check-milestone-closure.sh`
- `scripts/test-check-milestone-closure.sh`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/328-milestone-closure-audit-checklist.md`

## Why it exists

Trend workflow checks were validated by direct workflow inspection, but checker behavior itself was not fixture-tested.

Also, closure audit tracked trend workflow content (`M13-B/C`) but not whether naming-lock CI keeps the trend contract checks continuously enforced.

## What changed

1. Added workflow override to trend checker
- `test-benchmark-trend-workflow-contract.sh` now supports:
  - `--workflow <path>`

2. Added trend guard regression script
- New `test-benchmark-trend-workflow-contract-guard.sh` validates:
  - passing fixture,
  - failure when strict quality flag is removed,
  - failure when artifact upload contract is removed.

3. Wired trend checks into naming-lock CI
- `naming-lock.yml` now runs:
  - `scripts/test-benchmark-trend-workflow-contract.sh`
  - `scripts/test-benchmark-trend-workflow-contract-guard.sh`

4. Added closure gate `M13-E`
- `check-milestone-closure.sh` now verifies naming-lock includes trend contract and guard tests.
- `test-check-milestone-closure.sh` fixture coverage includes required `M13-E` gate presence.

## Validation

```bash
scripts/test-benchmark-trend-workflow-contract.sh
scripts/test-benchmark-trend-workflow-contract-guard.sh
scripts/test-check-milestone-closure.sh
scripts/test-roadmap-closure-gate-alignment.sh
scripts/check-milestone-closure.sh --fail-on-pending
scripts/check-naming-lock.sh
```

## Tradeoffs

- Adds two static guard scripts and one additional closure gate.
- Improves confidence that trend workflow policy checks remain behaviorally correct and continuously enforced.
