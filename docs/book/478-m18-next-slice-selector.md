# 478 M18 Next-Slice Selector

This chapter defines M18-S3: selecting the next closure-gated slice from kickoff + priority artifacts.

## 1) What it is

Script:

- `scripts/select-m18-next-slice.sh`

Contract test:

- `scripts/test-select-m18-next-slice.sh`

## 2) Why it exists

M18 now has two planning artifacts:

- kickoff summary,
- priority matrix.

The selector turns them into one explicit, machine-readable next-slice recommendation tied to a closure gate.

## 3) Inputs

- `--kickoff-json <path>`
- `--matrix-json <path>`
- `--output <path>`
- `--format <text|json>`

## 4) Behavior

Selection rules:

- If kickoff is failing (`overall != PASS`) or friction exists, force runtime remediation recommendation.
- Otherwise select based on top-priority matrix track (`editor`, `release`, `runtime`).

Output includes:

- `selectedTrack`,
- `recommendation.id`,
- `recommendation.title`,
- fixed closure gate `M18-C`.

Current recommendation id mapping:

- `editor` -> `M18-S4-editor-contract-expansion`
- `release` -> `M18-S5-release-publish-integrity-contract-expansion`
- `runtime` -> `M18-S6-runtime-confidence-hardening` (or `M18-S6-runtime-remediation-first` when kickoff is not `PASS`).

## 5) Verification

- `scripts/test-select-m18-next-slice.sh`
- `scripts/check-milestone-closure.sh --fail-on-pending`
