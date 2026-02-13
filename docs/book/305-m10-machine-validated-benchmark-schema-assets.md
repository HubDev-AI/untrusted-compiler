# 305 M10 Slice: Machine-Validated Benchmark Schema Assets

This chapter documents turning the benchmark artifact contract into machine-validated schema assets.

## What it is

Updated:
- `benchmark-suite/spec/artifact-contract-v0.1.md`
- `benchmark-suite/spec/schemas/*.schema.json`
- `benchmark-suite/scripts/testdata/sample-*.json`
- `scripts/check-naming-lock.sh`

## Why it exists

The contract markdown captured artifact names/keys, but validation logic lived as inline checks only. This slice makes schema requirements explicit and reusable as dedicated assets.

## How it works internally

1. Added schema files under `benchmark-suite/spec/schemas/` for:
   - report
   - summary
   - step-summary
   - compare-report
   - compare-matrix
   - analysis
   - step-matrix
   - artifact-manifest
2. Updated benchmark testdata coverage:
   - added sample compare matrix, analysis, and artifact manifest files,
   - aligned report samples with required `selectedEndpoints` field.
3. Reworked naming-lock benchmark checks:
   - validates sample artifacts against schema-required keys,
   - enforces schema version const where defined,
   - validates artifact-manifest item keys (`path`, `sha256`, `sizeBytes`).

## Inputs, outputs, and constraints

- Inputs:
  - schema assets in `benchmark-suite/spec/schemas/`
  - sample artifacts in `benchmark-suite/scripts/testdata/`
- Outputs:
  - deterministic pass/fail contract check in `scripts/check-naming-lock.sh`
- Constraints:
  - validation intentionally stays lightweight (required keys + version const), not full JSON Schema engine evaluation.

## Failure modes and diagnostics

- Missing schema asset:
  - naming-lock fails with `missing schema asset`.
- Sample JSON missing required key:
  - naming-lock reports exact key + sample file path.
- Version mismatch for versioned artifacts:
  - naming-lock reports expected schema version.

## Example usage

```bash
scripts/check-naming-lock.sh
benchmark-suite/scripts/test_preflight.sh
benchmark-suite/scripts/test_compare_matrix.sh
benchmark-suite/scripts/test_publish_report.sh
```

## Tradeoffs and next steps

- Tradeoff:
  - current checks use schema-required keys and selected nested constraints, not full JSON Schema validation semantics.
- Next:
  - add a standalone benchmark contract validator command to run schema checks independently from naming-lock.
