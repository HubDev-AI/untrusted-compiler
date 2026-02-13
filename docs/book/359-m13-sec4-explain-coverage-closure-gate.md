# M13 Slice: sec4 Explain-Coverage Closure Gate

This slice adds strict closure tracking for `sec4 explain` coverage enforcement in naming-lock CI.

## What it is

Updated:
- `scripts/check-milestone-closure.sh`
- `scripts/test-check-milestone-closure.sh`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/328-milestone-closure-audit-checklist.md`

## Why it exists

`check-sec4-explain-audit-coverage.sh` was already part of naming-lock, but strict closure did not explicitly verify that guard as a required milestone gate.

Adding an explicit closure gate keeps M13 explain-coverage guarantees visible and machine-enforced.

## What changed

1. Added closure gate `M13-F`
- `check-milestone-closure.sh` now verifies naming-lock workflow includes:
  - `scripts/check-sec4-explain-audit-coverage.sh`

2. Added fixture regression for missing explain checker
- `test-check-milestone-closure.sh` now includes a negative naming-lock fixture that omits the explain checker and expects strict closure failure.

3. Synced roadmap and closure checklist docs
- `docs/05-sec4-master-roadmap.md` closure table now includes `M13-F`.
- Historical closure-expansion notes now mention `M13-F`.
- `docs/book/328-milestone-closure-audit-checklist.md` now lists explain-checker enforcement in M13 closure checks.

## Validation

```bash
scripts/test-check-milestone-closure.sh
scripts/test-roadmap-closure-gate-alignment.sh
scripts/check-milestone-closure.sh --fail-on-pending
scripts/check-naming-lock.sh
```

## Tradeoffs

- Adds one more strict closure invariant tied to naming-lock workflow shape.
- Improves confidence that explain-map coverage enforcement cannot silently drop from CI while milestones still report PASS.
