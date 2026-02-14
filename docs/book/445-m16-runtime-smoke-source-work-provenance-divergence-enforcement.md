# 445 M16 Slice: Runtime-Smoke Source/Work Provenance Divergence Enforcement

This chapter documents a checker hardening rule that requires runtime-smoke source/work provenance values to differ.

## 1) What it is

`scripts/check-runtime-smoke-artifacts.sh` now rejects metadata where:

- `sourceProject == workProject`

The checker emits:

- `run-metadata.txt sourceProject and workProject must differ`

`scripts/test-check-runtime-smoke-artifacts.sh` now includes `bad-same-project` regression coverage for this branch.

## 2) Why it exists

`sourceProject` and `workProject` describe different provenance roles. If they collapse to the same value, runtime-smoke evidence can hide copy/run flow mistakes and reduce artifact traceability.

## 3) How it works internally

1. Checker validates non-empty source/work fields (existing behavior).
2. Checker extracts first matching values for both keys.
3. Checker compares values and fails if identical.
4. Regression fixture writes identical values and asserts deterministic failure message.

## 4) Inputs, outputs, constraints

Inputs:

- `run-metadata.txt` with source/work provenance fields

Outputs:

- pass/fail checker status with deterministic divergence diagnostic

Constraints:

- divergence check runs only after presence/non-empty checks
- first occurrence of each key is used for comparison

## 5) Failure modes and diagnostics

- if source/work values are identical:
  - checker exits non-zero with `run-metadata.txt sourceProject and workProject must differ`

## 6) Example usage

```bash
scripts/test-check-runtime-smoke-artifacts.sh
scripts/smoke-sec4-run-hello-api.sh --artifacts-dir /tmp/sec4-runtime-smoke
scripts/check-runtime-smoke-artifacts.sh --artifacts-dir /tmp/sec4-runtime-smoke
```

## 7) Trade-offs and next steps

Trade-offs:

- checker logic becomes slightly more semantic than pure key-presence validation.

Next steps:

- add optional normalization checks (for example disallow surrounding whitespace) if metadata format strictness is expanded.

## Verification

- `scripts/test-check-runtime-smoke-artifacts.sh`
- `scripts/smoke-sec4-run-hello-api.sh --artifacts-dir <tmp>`
- `scripts/check-runtime-smoke-artifacts.sh --artifacts-dir <tmp>`
