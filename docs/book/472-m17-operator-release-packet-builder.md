# 472 M17 Operator Release Packet Builder

This chapter adds a packet builder that captures operator handoff readiness artifacts in one output directory.

## 1) What it is

Script:

- `scripts/build-m17-operator-release-packet.sh`

Outputs:

- `readiness-summary.json`
- `closure-gates.json`
- `artifact-manifest.txt`
- `release-packet.json`

## 2) Why it exists

Teams need one reproducible package for review and handoff, not scattered outputs across multiple commands.

## 3) Inputs

- `--artifacts-root` (default: `build/operator-handoff-smoke`)
- `--output-dir` (default: `build/operator-release-packet`)

## 4) Behavior

The builder runs:

1. readiness summary in JSON mode,
2. strict closure gate snapshot in JSON mode,
3. SHA-256 manifest generation for all artifact files.

Then it writes packet metadata with overall summary/closure status and artifact count.

## 5) Verification

- `scripts/test-build-m17-operator-release-packet.sh`
- `scripts/check-milestone-closure.sh --fail-on-pending`
