# M9 Slice: Release Gate Milestone Closure Enforcement

This slice ties alpha release readiness to strict milestone closure evidence.

It also feeds closure-audit gate `M9-D`, which verifies that the release gate keeps strict closure enforcement wired in.

## What it is

Updated:
- `scripts/release-alpha-gate.sh`
- `scripts/verify-release-promotion-inputs.sh`

## Why it exists

Alpha release automation already enforced naming lock and artifact identity consistency, but it did not require current milestone closure evidence to be satisfied at release time.

That left a drift risk: release artifacts could be produced even when closure evidence (M10/M13) was stale.

## What changed

1. Release gate now enforces strict closure
- `release-alpha-gate.sh` now runs:
  - `scripts/check-milestone-closure.sh --fail-on-pending`

2. Release summary now stamps closure status
- `summary.txt` now includes:
  - `milestone closure: PASS`

3. Promotion verifier now checks closure stamp
- `verify-release-promotion-inputs.sh` now requires:
  - `milestone closure` summary field equals `PASS`.

## Validation

```bash
scripts/test-verify-release-promotion-inputs.sh
scripts/test-generate-release-publish-manifest.sh
scripts/test-verify-release-publish-manifest.sh
```

`test-verify-release-promotion-inputs.sh` now also asserts verifier failure when the `milestone closure` summary stamp is removed.

## Tradeoffs

- Release-gate strictness increases and can block releases when closure evidence drifts.
- This is intentional: release artifacts should only be produced when roadmap closure claims are still true.
