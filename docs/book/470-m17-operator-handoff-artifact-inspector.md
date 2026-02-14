# 470 M17 Operator Handoff Artifact Inspector

This chapter adds a deterministic artifact-inspection helper for operator handoff outputs.

## 1) What it is

Script:

- `scripts/inspect-m17-operator-handoff-artifacts.sh`

It validates branch artifacts via the runtime-smoke bundle checker and prints a branch summary in text or JSON.

## 2) Why it exists

After CI/local handoff runs, operators need a fast way to inspect `default` and `max-body` artifacts without manually opening every file.

## 3) Inputs and outputs

Inputs:

- `--artifacts-root` (default: `build/operator-handoff-smoke`)
- `--format text|json` (default: `text`)

Outputs:

- text summary:
  - branch name
  - port/timeout/maxBody/runFlags
  - health/users status lines
- JSON summary:
  - `version`
  - `artifactsRoot`
  - `indexPath`
  - `branchCount`
  - `branches[]`

## 4) Failure behavior

Fails on:

- missing branch artifacts,
- malformed bundle shape (via `check-runtime-smoke-bundle.sh`),
- missing `jq`,
- unsupported output format.

## 5) Verification

- `scripts/test-inspect-m17-operator-handoff-artifacts.sh`
- `scripts/check-milestone-closure.sh --fail-on-pending`
