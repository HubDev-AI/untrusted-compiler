# 307 M10 Slice: Standalone Benchmark Contract Validator

This chapter documents a standalone benchmark schema validator command and CI smoke coverage.

## What it is

Updated:
- `benchmark-suite/scripts/validate_contract_schema.sh`
- `benchmark-suite/scripts/test_validate_contract_schema.sh`
- `.github/workflows/benchmark-smoke.yml`
- `docs/05-sec4-master-roadmap.md`

## Why it exists

Schema checks were previously coupled to naming-lock. This slice makes benchmark contract validation independently runnable and CI-visible as a dedicated smoke step.

## How it works internally

`validate_contract_schema.sh` validates benchmark sample artifacts against schema assets under `benchmark-suite/spec/schemas/`:
- report samples (`sample-<impl>-report.json`)
- summary samples (`sample-summary-*.json`)
- step summary/matrix samples
- compare matrix sample
- analysis sample
- artifact manifest sample

Validation behavior:
1. required top-level keys,
2. schema version const checks where defined,
3. artifact-manifest item required keys,
4. report sample `impl` value alignment with filename,
5. compare-matrix row-shape checks for `compared[]` + `leader` (`loadGenerator`, `constantRate`, and core row fields).

## Tests and CI wiring

- `test_validate_contract_schema.sh`:
  - verifies validator passes on current fixtures,
  - mutates a summary sample to remove a required key and verifies validator fails,
  - mutates compare-matrix leader row to remove `constantRate` and verifies validator fails.
- `benchmark-smoke.yml` now runs:
  - `benchmark-suite/scripts/test_validate_contract_schema.sh`
  - then existing benchmark smoke tests.

## Inputs, outputs, and constraints

- Inputs:
  - `benchmark-suite/spec/schemas/*.schema.json`
  - `benchmark-suite/scripts/testdata/sample-*.json`
- Output:
  - pass/fail contract validation status for benchmark artifact samples.
- Constraint:
  - validator intentionally enforces required keys/version constraints only (lightweight, deterministic).

## Example usage

```bash
benchmark-suite/scripts/validate_contract_schema.sh
benchmark-suite/scripts/test_validate_contract_schema.sh
```

## Tradeoffs and next steps

- Tradeoff:
  - not a full JSON Schema engine evaluation of all nested types.
- Next:
  - extend validator coverage to generated benchmark outputs in scoped dry-run/full-suite CI workflows.
