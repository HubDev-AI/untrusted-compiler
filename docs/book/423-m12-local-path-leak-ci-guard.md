# 423 M12 Slice: Local Path Leak CI Guard

This chapter documents a naming-hygiene guard that prevents local absolute path leakage into repository files.

## What it is

A new guard script plus CI test wiring:

- `scripts/check-no-local-path-leaks.sh`
- `scripts/test-check-no-local-path-leaks.sh`
- naming-lock workflow step running the guard test.

The checker fails when tracked files contain the forbidden local workspace prefix token used on this machine.

## Why it exists

Project history includes a hard requirement to never commit local machine path roots. This guard turns that requirement into automated enforcement.

## Implementation details

1. Added checker script with configurable `--repo-root`.
2. Checker scans repository content (excluding build/cache folders) for forbidden absolute path token.
3. Added guard test script with pass/fail fixtures.
4. Wired test into `.github/workflows/naming-lock.yml`.

## Validation

Executed locally:

- `scripts/test-check-no-local-path-leaks.sh`

Result: pass.

## Tradeoffs and next steps

- Guard is intentionally strict for the known local prefix token and catches accidental leaks early.
- If team environments expand, token list can be generalized to a policy-driven denylist while preserving deterministic CI behavior.
