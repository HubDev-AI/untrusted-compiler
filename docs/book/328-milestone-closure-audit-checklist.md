# 328 Milestone Closure Audit Checklist

This chapter defines the strict closure audit for milestone completion claims.

## What it is

Updated:
- `scripts/check-milestone-closure.sh`
- `scripts/test-check-milestone-closure.sh`
- `docs/05-sec4-master-roadmap.md`

This adds an evidence-based closure checker for the highest-risk late milestones (M9/M10/M13).

## Why it exists

Implementation progress and milestone closure are not the same. This checklist prevents us from claiming completion when required live evidence is still missing.

## Closure checks (current scope)

`check-milestone-closure.sh` verifies:

1. M9 release hardening foundations
- release gate script exists,
- release gate workflow exists,
- promotion verifier + publish manifest verifier chain exists.

2. M10 live comparison evidence
- compare matrix exists and includes implementation IDs:
  - `sec4`
  - `go`
  - `node`
  - `rust`

3. M13 live trend evidence
- trend-note chapter contains at least one:
  - `## Trend Entry (YYYY-MM-DD)`

## Example usage

```bash
scripts/check-milestone-closure.sh
scripts/check-milestone-closure.sh --fail-on-pending
scripts/test-check-milestone-closure.sh
```

## Current result (2026-02-13)

- M9 gate checks: PASS
- M10 live cross-impl evidence: PASS
- M13 live trend-note evidence: PASS

## Inputs, outputs, and constraints

- Inputs:
  - repository files and optional matrix/trend-note paths.
- Output:
  - structured PASS/PENDING table and overall status.
- Constraints:
  - the checker validates artifact presence/content shape, not benchmark quality itself.

## Tradeoffs and next steps

- Tradeoff:
  - strict checks intentionally focus on late-stage closure evidence and do not re-audit all early milestones.
- Next:
  - keep closure evidence refreshed via `scripts/refresh-closure-evidence-from-ci.sh` after new benchmark or trend runs.
