# 296 M12 Slice: Naming-Lock CI Workflow

This chapter documents CI enforcement for the naming-lock contract.

## What it is

Added:
- `.github/workflows/naming-lock.yml`

Updated:
- `docs/05-sec4-master-roadmap.md`
- `docs/book/210-release-notes-and-compatibility.md`

## Why it exists

`scripts/check-naming-lock.sh` protects naming consistency locally and in release gating, but drift should be blocked at pull-request time too. A dedicated CI workflow catches regressions early.

## How it works internally

1. Trigger on:
   - `pull_request`
   - `push` to `main`
2. Checkout repository.
3. Run `scripts/check-naming-lock.sh`.
4. Fail the workflow if legacy naming appears or required naming-contract tokens are missing.

## Inputs, outputs, and constraints

- Input: tracked repo files scanned by `scripts/check-naming-lock.sh`.
- Output: pass/fail CI signal for naming lock compliance.
- Constraint: workflow intentionally runs only naming checks; release artifact validation remains in `alpha-release-gate.yml`.

## Failure modes and diagnostics

- Legacy token detected:
  - workflow fails,
  - script output includes pattern and file+line matches.
- Required token missing:
  - workflow fails with explicit missing-token message.

## Example usage

Run locally before push:

```bash
scripts/check-naming-lock.sh
```

## Tradeoffs and next steps

- Tradeoff:
  - duplicates part of alpha gate validation by design for faster PR feedback.
- Next:
  - optionally add branch protection requiring this workflow before merge.
