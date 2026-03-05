# M37 Alpha.2 Release Closure Refresh

This chapter records the post-M37 closure refresh that produced the second alpha tag from the current `dev` baseline and revalidated the publish chain.

## What changed

1. Stabilized release-gate execution under full test load:
   - one-shot C harness helpers now switch accepted sockets back to blocking mode before request-read loops, removing intermittent `WouldBlock` read failures.
   - `scripts/release-alpha-gate.sh` now runs cargo tests in deterministic serialized mode by default:
     - `CARGO_BUILD_JOBS=1`
     - `RUST_TEST_THREADS=1`
   - both settings remain overrideable via:
     - `SEC4_RELEASE_GATE_CARGO_BUILD_JOBS`
     - `SEC4_RELEASE_GATE_TEST_THREADS`
2. Refreshed naming-lock compatibility docs by removing repo-local absolute paths from `docs/codex-operator-handoff.md`.
3. Rebuilt the M37 decision artifact chain from refreshed gate evidence:
   - `build/m37-alpha-publish-checklist-delta.json`
   - `build/m37-alpha-release-notes.json`
   - `build/m37-alpha-tag-decision-record.json`

## Tag + verification evidence

1. Created and pushed:
   - `v0.1.0-alpha.2`
2. Ran post-tag verifier chain:
   - `scripts/release-alpha-gate.sh --skip-tests`
   - `scripts/verify-release-promotion-inputs.sh`
   - `scripts/generate-release-publish-manifest.sh`
   - `scripts/verify-release-publish-manifest.sh`
3. Final verifier status:
   - promotion inputs: `PASS`
   - publish manifest: `PASS`

## Outcome

Alpha release-closure steps are refreshed and executable on current `dev`, with `v0.1.0-alpha.2` as the latest tagged baseline and publish-manifest verification chain passing.
