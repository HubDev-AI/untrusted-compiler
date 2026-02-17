# M37-S10 Full Alpha Gate Execution With Tests

## What it is

M37-S10 executes the full alpha release gate (including full `cargo test`) and revalidates the M37 decision chain from the resulting artifacts.

## Why it exists

A `--skip-tests` gate pass is useful for rapid checks, but alpha-tag readiness requires the real full-gate path with test coverage enabled.

## How it works

1. Executed full release gate:
   - `scripts/release-alpha-gate.sh`
2. Rebuilt decision artifacts from the refreshed full-gate outputs:
   - `scripts/build-m37-alpha-publish-checklist-delta.sh --format json > build/m37-alpha-publish-checklist-delta.json`
   - `scripts/build-m37-alpha-release-notes.sh --format json > build/m37-alpha-release-notes.json`
   - `scripts/build-m37-alpha-tag-decision-record.sh --decision go --format json > build/m37-alpha-tag-decision-record.json`
3. Verified decision continuity:
   - checklist: `PASS GO`
   - release notes: `alpha-ready GO`
   - tag decision record: `GO PASS`

## Validation

- `scripts/release-alpha-gate.sh`
- `scripts/build-m37-alpha-publish-checklist-delta.sh --format json`
- `scripts/build-m37-alpha-release-notes.sh --format json`
- `scripts/build-m37-alpha-tag-decision-record.sh --decision go --format json`
- `scripts/test-build-m37-alpha-publish-checklist-delta.sh`
- `scripts/test-build-m37-alpha-release-notes.sh`
- `scripts/test-build-m37-alpha-tag-decision-record.sh`

## Outcome

- Full alpha gate currently passes end-to-end with tests enabled.
- Decision chain remains stable at `GO`.
- Next operational step is alpha tag execution plus post-tag verification, followed by `WASM_START_GATE` opening.
