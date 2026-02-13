# M13 Slice: Closure Audit JSON Output Contract

This slice adds machine-readable output mode for milestone closure auditing.

## What it is

Updated:
- `scripts/check-milestone-closure.sh`
- `scripts/test-check-milestone-closure.sh`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/328-milestone-closure-audit-checklist.md`

## Why it exists

Closure audit output was text-only, which limited deterministic CI/automation consumption.

A JSON mode provides stable fields for downstream tooling while preserving existing text mode for operators.

## What changed

1. Added `--format` option
- `check-milestone-closure.sh` now supports:
  - `--format text` (default)
  - `--format json`

2. Added stable JSON payload
- JSON output includes:
  - `repo`
  - `overall`
  - `message`
  - `pendingCount`
  - `gates[]` with fields:
    - `gate`
    - `status`
    - `check`
    - `evidence`

3. Kept path-leak hardening in JSON mode
- Evidence paths remain repo-relative when possible.
- `repo` emits `.` for current workspace root.

4. Expanded fixture tests
- `test-check-milestone-closure.sh` now validates JSON mode for passing fixtures:
  - `overall=PASS`
  - `pendingCount=0`
  - required gate IDs present (`M9-H`, `M13-D`)
  - no absolute temp-path leakage.

## Validation

```bash
scripts/test-check-milestone-closure.sh
scripts/test-roadmap-closure-gate-alignment.sh
scripts/check-milestone-closure.sh --format json --fail-on-pending
```

## Tradeoffs

- JSON rendering adds small runtime overhead from `jq` composition.
- Text mode remains unchanged for existing human/operator workflows.
