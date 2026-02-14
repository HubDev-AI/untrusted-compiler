# 477 M18 Priority Matrix Artifact

This chapter defines M18-S2: deterministic prioritization across runtime, editor, and release tracks.

## 1) What it is

Script:

- `scripts/build-m18-priority-matrix.sh`

Contract test:

- `scripts/test-build-m18-priority-matrix.sh`

## 2) Why it exists

M18 needs a stable way to pick the next slices after handoff evidence is available. A deterministic matrix prevents ad-hoc reprioritization drift.

## 3) Inputs

- `--report <path>` (rehearsal report)
- `--output-json <path>`
- `--output-markdown <path>`
- `--format <text|json>`

## 4) Behavior

The matrix derives scores from:

- rehearsal `overall` status,
- `friction` count.

Ordering contract:

- PASS + no friction: `editor` > `release` > `runtime`
- FAIL (or friction): `runtime` > `release` > `editor`

## 5) Outputs

- JSON matrix with sorted tracks and explicit priorities.
- Markdown table + rationale list for operator planning docs.

## 6) Verification

- `scripts/test-build-m18-priority-matrix.sh`
- `scripts/check-milestone-closure.sh --fail-on-pending`
