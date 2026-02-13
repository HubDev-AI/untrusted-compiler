# 229 M10 Slice: Cross-Implementation Contract Parity Smoke

This chapter documents cross-implementation service contract smoke validation for M10 comparators.

## What it is

Added:
- `benchmark-suite/scripts/test_service_contracts.sh`
- `benchmark-suite/scripts/test_test_service_contracts.sh`

Updated:
- `benchmark-suite/Makefile` (`test-services` target, script-test wiring)
- `benchmark-suite/README.md`

The parity script runs smoke checks across selected implementations (`sec4,node,go,rust,c`) to ensure endpoint behavior remains aligned.

## Why it exists

M10 fairness depends on comparable service behavior. This slice adds a single parity gate that catches contract drift across implementations before benchmark runs.

## How it works internally

1. Validates requested implementation list.
2. Resolves each implementation’s smoke script:
   - `services/<impl>/smoke.sh`
3. Runs each smoke script sequentially (shared port `8080`) and fails fast on the first error.
4. Supports `--dry-run` for command-plan validation.

## Inputs, outputs, and constraints

- Inputs:
  - implementation list (`--impls`, default all comparators).
- Outputs:
  - pass/fail shell status,
  - per-implementation smoke output logs.
- Constraints:
  - sequential execution required due shared local port.

## Failure modes and diagnostics

- unsupported implementation -> explicit error.
- missing/non-executable smoke script -> explicit error.
- any smoke failure -> non-zero exit with per-impl context.

## Example usage

```bash
make -C benchmark-suite test-services
```

Dry run:

```bash
benchmark-suite/scripts/test_service_contracts.sh --dry-run --impls sec4,node
```

## Tradeoffs and next steps

- Tradeoff:
  - parity checks are smoke-level and contract-oriented, not full workload/perf validation.
- Next:
  - add shared golden response fixtures and strict body-shape assertions for deeper parity checks,
  - integrate parity status into benchmark report metadata.
