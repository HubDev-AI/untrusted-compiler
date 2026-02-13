# 297 M12 Slice: Benchmark Implementation-ID Naming Guard

This chapter documents extending naming-lock enforcement into benchmark implementation IDs.

## What it is

Updated:
- `scripts/check-naming-lock.sh`
- `docs/05-sec4-master-roadmap.md`

## Why it exists

M12 naming lock must cover not only language/docs/CLI/editor surfaces but also benchmark identities. Legacy benchmark IDs can silently skew reports and comparisons.

## How it works internally

`scripts/check-naming-lock.sh` now validates benchmark naming contract:

1. Required implementation directories exist:
   - `benchmark-suite/services/sec4`
   - `benchmark-suite/services/go`
   - `benchmark-suite/services/node`
   - `benchmark-suite/services/rust`
   - `benchmark-suite/services/c`
2. Legacy `benchmark-suite/services/ailang` directory is rejected if present.
3. Benchmark JSON testdata `impl` values are parsed and constrained to:
   - `sec4`, `go`, `node`, `rust`, `c`

## Inputs, outputs, and constraints

- Input: benchmark service directory names and `benchmark-suite/scripts/testdata/*.json`.
- Output: deterministic pass/fail contract check.
- Constraint: check currently validates implementation IDs, not full benchmark artifact schema naming.

## Failure modes and diagnostics

- Missing required implementation directory:
  - emits explicit missing-path error.
- Legacy/unsupported implementation ID:
  - emits explicit unsupported-ID error with offending value.

## Example usage

```bash
scripts/check-naming-lock.sh
```

## Tradeoffs and next steps

- Tradeoff:
  - implementation-ID guard is strict; adding new benchmark implementations requires updating the allowlist.
- Next:
  - add artifact filename/schema-key guardrails after final M10 benchmark output schema freeze.
