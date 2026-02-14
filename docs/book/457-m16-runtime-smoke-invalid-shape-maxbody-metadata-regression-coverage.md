# 457 M16 Slice: Runtime-Smoke Invalid-Shape maxBody Metadata Regression Coverage

This chapter documents adding explicit regression coverage for invalid-shape `maxBodyBytes` metadata values.

## 1) What it is

`scripts/test-check-runtime-smoke-artifacts.sh` now includes a fixture with:

- `maxBodyBytes=abc`

Expected checker failure:

- `run-metadata.txt missing or invalid maxBodyBytes field`

## 2) Why it exists

Existing maxBody coverage already handled missing fields, non-positive numeric values, and log-correlation mismatches. This slice fills the remaining gap for non-numeric shape violations.

## 3) How it works internally

1. Clone the passing fixture set.
2. Rewrite metadata with non-numeric `maxBodyBytes`.
3. Run checker and assert non-zero exit.
4. Assert deterministic invalid-shape diagnostic.

## 4) Inputs, outputs, constraints

Inputs:

- checker regression fixture metadata

Outputs:

- deterministic invalid-shape diagnostic assertion

Constraints:

- fixture must retain all other required metadata and artifact fields so failure isolates maxBody shape

## 5) Failure modes and diagnostics

- non-numeric `maxBodyBytes`:
  - `run-metadata.txt missing or invalid maxBodyBytes field`

## 6) Example usage

```bash
scripts/test-check-runtime-smoke-artifacts.sh
```

## 7) Trade-offs and next steps

Trade-offs:

- checker regression matrix grows as metadata contract branches expand.

Next steps:

- add explicit overflow-size fixtures if maxBody metadata needs upper-bound enforcement aligned with runtime limits.

## Verification

- `scripts/test-check-runtime-smoke-artifacts.sh`
- `scripts/check-runtime-smoke-artifacts.sh --artifacts-dir <tmp>`
