# 439 M16 Slice: Runtime-Smoke Checker runFlags Regression Fixture

This chapter documents adding explicit regression coverage for missing `runFlags` metadata in runtime-smoke artifact checker tests.

## 1) What it is

`scripts/test-check-runtime-smoke-artifacts.sh` now includes a dedicated `bad-runflags` fixture where `run-metadata.txt` omits:

- `runFlags=--port,--oneshot,--serve-timeout-ms`

## 2) Why it exists

`M16-S32` added checker validation for `runFlags`, but regression coverage only had a missing-`oneshot` metadata scenario. A dedicated missing-`runFlags` fixture makes contract intent explicit and guards against accidental checker/test drift.

## 3) How it works internally

1. Clone the passing metadata fixture into `bad-runflags`.
2. Rewrite metadata without `runFlags`.
3. Run checker and assert failure.
4. Assert deterministic diagnostic text:
   - `run-metadata.txt missing deterministic runFlags field`

## 4) Inputs, outputs, constraints

Inputs:

- synthetic artifact directories in checker regression script

Outputs:

- deterministic pass/fail for missing runFlags metadata case

Constraints:

- checker diagnostic wording is part of this regression contract.

## 5) Failure modes and diagnostics

- if checker stops validating runFlags metadata, guard scenario unexpectedly passes and test fails.
- if checker message changes, regression script fails on missing expected diagnostic token.

## 6) Example usage

```bash
scripts/test-check-runtime-smoke-artifacts.sh
```

## 7) Trade-offs and next steps

Trade-offs:

- more fixture cases increase script length slightly.

Next steps:

- if metadata schema grows, consider table-driven fixture generation to reduce duplication in checker regression tests.

## Verification

- `scripts/test-check-runtime-smoke-artifacts.sh`
- `scripts/smoke-sec4-run-hello-api.sh --artifacts-dir <tmp>`
- `scripts/check-runtime-smoke-artifacts.sh --artifacts-dir <tmp>`
