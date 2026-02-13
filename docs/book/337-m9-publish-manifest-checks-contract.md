# M9 Slice: Publish Manifest Checks Contract

This slice extends publish-manifest metadata to carry release readiness checks and enforces them in manifest verification.

## What it is

Updated:
- `scripts/generate-release-publish-manifest.sh`
- `scripts/verify-release-publish-manifest.sh`
- `scripts/test-generate-release-publish-manifest.sh`
- `scripts/test-verify-release-publish-manifest.sh`

## Why it exists

Release summary already stamped `naming lock` and `milestone closure`, but publish manifest did not carry these values.

Downstream publish tooling consuming only `publish-manifest.json` could miss those readiness checks.

## What changed

1. Manifest includes `checks` section
- `checks.namingLock`
- `checks.milestoneClosure`
- values sourced from `summary.txt` during manifest generation.

2. Manifest verification enforces checks
- verifies manifest `checks` values match release summary,
- requires both checks to be `PASS`.

3. Tests expanded
- generation test now asserts checks fields exist and are `PASS`,
- generation test fails when summary milestone-closure entry is removed,
- verification test fails when manifest `checks.milestoneClosure` is tampered.

## Validation

```bash
scripts/test-generate-release-publish-manifest.sh
scripts/test-verify-release-publish-manifest.sh
```

## Tradeoffs

- Adds one more strict contract layer for publish tooling.
- Intentionally fails fast when release-readiness metadata drifts.
