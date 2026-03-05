# 473 M17 Operator Handoff Final Playbook

This chapter closes M17 with one ordered playbook for local validation, CI parity checks, and release-packet assembly.

## What it is

A canonical four-bundle command playbook:

- Bundle A: local validation flow.
- Bundle B: CI smoke + artifact inspection flow.
- Bundle C: release packet assembly flow.
- Bundle D: zed extension operator readiness flow.

## Why it exists

Prior slices introduced separate scripts. This playbook removes sequencing ambiguity and gives operators one deterministic path from local verification to handoff packet generation.

## Bundle A: Local Validation Flow

Run in order:

1. `scripts/check-m17-operator-handoff-readiness.sh --repo-root .`
2. `scripts/run-m17-operator-handoff-quickstart.sh --repo-root . --project examples/hello-api --artifacts-root build/runtime-smoke`
3. `scripts/summarize-m17-operator-handoff-readiness.sh --repo-root . --artifacts-root build/operator-handoff-smoke --format text`

Expected outcomes:

- readiness contracts pass,
- quickstart exits with `m17 operator handoff quickstart passed`,
- readiness summary reports `closureOverall: PASS`.

## Bundle B: CI Smoke and Artifact Inspection Flow

Run in order:

1. `scripts/run-m17-operator-handoff-ci-smoke.sh --repo-root . --artifacts-root build/operator-handoff-smoke`
2. `scripts/inspect-m17-operator-handoff-artifacts.sh --repo-root . --artifacts-root build/operator-handoff-smoke --format text`
3. `scripts/check-milestone-closure.sh --repo-root . --fail-on-pending`

Expected outcomes:

- both `default` and `max-body` artifact branches are present,
- inspector reports deterministic HTTP status evidence for health/users,
- strict closure audit stays PASS.

## Bundle C: Release Packet Assembly Flow

Run in order:

1. `scripts/build-m17-operator-release-packet.sh --repo-root . --artifacts-root build/operator-handoff-smoke --output-dir build/operator-release-packet`
2. `scripts/summarize-m17-operator-handoff-readiness.sh --repo-root . --artifacts-root build/operator-handoff-smoke --format json`
3. `scripts/check-milestone-closure.sh --repo-root . --format json --fail-on-pending`

Expected release packet outputs:

- `build/operator-release-packet/release-packet.json`
- `build/operator-release-packet/readiness-summary.json`
- `build/operator-release-packet/closure-gates.json`
- `build/operator-release-packet/artifact-manifest.txt`

## Bundle D: Zed Extension Operator Readiness

Run in order:

1. `scripts/check-zed-extension-operator-readiness.sh --stage-output build/zed-extension-release-operator`
2. `scripts/summarize-m17-operator-handoff-readiness.sh --repo-root . --artifacts-root build/operator-handoff-smoke --format text`

Expected outcomes:

- `scripts/check-zed-extension-operator-readiness.sh` completes successfully,
- zed extension operator bundle manifest is present as:
  - `build/zed-extension-release-operator/bundle-manifest.json`.

## Verification

- `scripts/test-check-m17-operator-handoff-playbook.sh`
- `scripts/check-milestone-closure.sh --fail-on-pending`
