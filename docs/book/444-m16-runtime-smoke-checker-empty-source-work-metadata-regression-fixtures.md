# 444 M16 Slice: Runtime-Smoke Checker Empty Source/Work Metadata Regression Fixtures

This chapter documents regression coverage for empty provenance metadata values in runtime-smoke artifact validation.

## 1) What it is

`scripts/test-check-runtime-smoke-artifacts.sh` now includes two additional failing fixtures:

- `bad-source-empty` with `sourceProject=`
- `bad-work-empty` with `workProject=`

Both cases must fail checker validation with deterministic diagnostics.

## 2) Why it exists

Checker uses non-empty regex contracts (`^key=.+$`), but missing-key fixtures alone do not prove empty-value enforcement. This slice locks that behavior explicitly.

## 3) How it works internally

1. Copy passing fixture set.
2. Rewrite metadata to include an empty source/work value.
3. Run checker and assert non-zero exit.
4. Assert diagnostic contracts remain:
   - `run-metadata.txt missing sourceProject field`
   - `run-metadata.txt missing workProject field`

## 4) Inputs, outputs, constraints

Inputs:

- runtime-smoke checker script
- fixture metadata variants with empty values

Outputs:

- deterministic regression signals for empty-value provenance failures

Constraints:

- diagnostics intentionally match missing-field branches
- fixture structure must still include all required artifact files

## 5) Failure modes and diagnostics

- if checker regex relaxes to allow empty values:
  - regression tests fail because checker exits successfully unexpectedly.
- if error text changes:
  - regression tests fail on diagnostic token mismatch.

## 6) Example usage

```bash
scripts/test-check-runtime-smoke-artifacts.sh
scripts/smoke-sec4-run-hello-api.sh --artifacts-dir /tmp/sec4-runtime-smoke
scripts/check-runtime-smoke-artifacts.sh --artifacts-dir /tmp/sec4-runtime-smoke
```

## 7) Trade-offs and next steps

Trade-offs:

- checker test suite grows with more fixture branches as metadata contract tightens.

Next steps:

- add malformed-key-format fixtures (for example whitespace around `=`) if metadata parser strictness is tightened further.

## Verification

- `scripts/test-check-runtime-smoke-artifacts.sh`
- `scripts/smoke-sec4-run-hello-api.sh --artifacts-dir <tmp>`
- `scripts/check-runtime-smoke-artifacts.sh --artifacts-dir <tmp>`
