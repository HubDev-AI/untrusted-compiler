# M37-S7 Alpha Release Notes Package

## What it is

M37-S7 adds a deterministic alpha release-note package builder:

- `scripts/build-m37-alpha-release-notes.sh`
- `build/m37-alpha-release-notes.json`
- `build/m37-alpha-release-notes.md`

The builder consumes the M37-S6 checklist artifact and emits a single release-state summary (`alpha-ready` or `alpha-hold`) with identity hashes, checklist outcome, and known limits.

## Why it exists

After checklist evidence is computed, alpha promotion still needs human-readable release notes plus machine-readable summary metadata.

This slice guarantees the release-note package is reproducible from the checklist contract and does not depend on ad-hoc manual notes.

## How it works

1. Validate input checklist contract (`marker == M37-S6` and required identity/checklist fields).
2. Derive release state:
   - `alpha-ready` when checklist is `PASS/GO`,
   - `alpha-hold` otherwise.
3. Emit canonical package:
   - baseline statement,
   - checklist summary,
   - identity hashes,
   - known-limit list,
   - next action.

## Validation

- `bash -n scripts/build-m37-alpha-release-notes.sh`
- `bash -n scripts/test-build-m37-alpha-release-notes.sh`
- `scripts/test-build-m37-alpha-release-notes.sh`
- `scripts/build-m37-alpha-publish-checklist-delta.sh --format json > build/m37-alpha-publish-checklist-delta.json`
- `scripts/build-m37-alpha-release-notes.sh --format json`

## Trade-offs

- The release-note package reflects checklist evidence and does not replace release-gate execution itself.
- Known limits are intentionally explicit so alpha readiness is not overstated.

## Next

1. Execute final alpha tag decision checklist using M37-S6 + M37-S7 artifacts.
2. Record closure outcome for M37 in roadmap/book.
