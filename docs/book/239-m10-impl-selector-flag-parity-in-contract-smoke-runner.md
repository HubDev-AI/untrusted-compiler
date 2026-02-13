# 239 M10 Slice: Impl-Selector Flag Parity in Contract Smoke Runner

This chapter documents argument-parsing parity updates for implementation selection in benchmark smoke tooling.

## What it is

Updated:
- `benchmark-suite/scripts/test_service_contracts.sh`
- `benchmark-suite/scripts/test_test_service_contracts.sh`

Key changes:
- `test_service_contracts.sh` now supports both:
  - `--impls node,c`
  - `--impls=node,c`
- test coverage added for `--impls=...` path.

## Why it exists

Matrix/preflight scripts already accepted both argument styles. Keeping contract-smoke runner aligned reduces command friction and avoids surprising parser inconsistencies in CI/manual use.

## How it works internally

1. Added case branch for `--impls=*` in smoke-runner argument parser.
2. Existing implementation validation logic is reused unchanged.
3. Test asserts dry-run output for `--impls=node,c` includes expected smoke scripts.

## Inputs, outputs, and constraints

- Inputs:
  - implementation selector argument.
- Outputs:
  - same smoke command plan/execution behavior as space-separated form.
- Constraints:
  - implementation names must still pass supported-set validation.

## Failure modes and diagnostics

- unknown impl in either form still fails deterministically with validation error.

## Example usage

```bash
benchmark-suite/scripts/test_service_contracts.sh --dry-run --impls=node,c
```

## Tradeoffs and next steps

- Tradeoff:
  - none; parser compatibility only.
- Next:
  - keep option-format parity across all benchmark-suite CLI scripts as new flags are added.
