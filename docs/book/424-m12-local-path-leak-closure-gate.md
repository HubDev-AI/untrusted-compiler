# 424 M12 Slice: Local Path Leak Closure Gate

This chapter documents strict-closure integration for the local absolute path leak guard introduced in M12.

## What it is

A new strict closure gate was added:

- `M12-B` in `scripts/check-milestone-closure.sh`

The gate requires naming-lock CI to execute:

- `scripts/test-check-no-local-path-leaks.sh`

## Why it exists

The guard script and workflow step already prevented local path leaks, but closure audit did not enforce that contract. `M12-B` closes that gap and makes the requirement auditable alongside other milestone gates.

## Implementation details

1. Added `bool_has_local_path_leak_ci_guard` detection in closure script.
2. Added `emit_check("M12-B", ...)` with naming-lock workflow evidence.
3. Updated closure fixture test to include local-path guard step and assert `M12-B` in deterministic gate ordering.
4. Updated roadmap strict-closure table and interpretation bullets to include `M12-B`.

## Validation

Executed locally:

- `scripts/test-check-milestone-closure.sh`
- `scripts/test-roadmap-closure-gate-alignment.sh`
- `scripts/check-milestone-closure.sh --format json --fail-on-pending`

Result: pass.

## Tradeoffs and next steps

- Closure surface is slightly larger, but this preserves naming hygiene as a non-regressible contract.
- If additional path-leak denylist tokens are introduced later, `M12-B` can remain stable while checker internals expand.
