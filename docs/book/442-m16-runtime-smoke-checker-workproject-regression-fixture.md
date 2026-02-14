# 442 M16 Slice: Runtime-Smoke Checker workProject Regression Fixture

This chapter documents adding explicit regression coverage for missing `workProject` metadata in runtime-smoke artifact validation.

## 1) What it is

`scripts/test-check-runtime-smoke-artifacts.sh` now contains a `bad-work` fixture where `run-metadata.txt` omits `workProject`.

The checker is expected to fail with:

- `run-metadata.txt missing workProject field`

## 2) Why it exists

M16-S37 added checker enforcement for both `sourceProject` and `workProject`, but regression coverage only pinned the missing-`sourceProject` branch. This slice adds symmetric coverage so both provenance keys are locked by tests.

## 3) How it works internally

1. Copy the passing fixture directory to `bad-work`.
2. Rewrite `run-metadata.txt` without `workProject`.
3. Execute checker and assert non-zero exit.
4. Assert stderr contains deterministic missing-field diagnostic.

## 4) Inputs, outputs, constraints

Inputs:

- artifact checker script
- synthetic runtime-smoke fixture directories

Outputs:

- deterministic regression signal for missing `workProject`

Constraints:

- diagnostic text is contract-locked and used by guard workflows
- fixture shape must continue matching checker-required artifact files

## 5) Failure modes and diagnostics

- if checker stops enforcing `workProject`, regression test fails because it expects non-zero checker exit.
- if checker message drifts, regression test fails on missing expected diagnostic token.

## 6) Example usage

```bash
scripts/test-check-runtime-smoke-artifacts.sh
scripts/smoke-sec4-run-hello-api.sh --artifacts-dir /tmp/sec4-runtime-smoke
scripts/check-runtime-smoke-artifacts.sh --artifacts-dir /tmp/sec4-runtime-smoke
```

## 7) Trade-offs and next steps

Trade-offs:

- adds another metadata fixture case to maintain when checker contracts evolve.

Next steps:

- extend runtime-smoke checker coverage with fixture-level checks for malformed metadata formats, not only missing keys.

## Verification

- `scripts/test-check-runtime-smoke-artifacts.sh`
- `scripts/smoke-sec4-run-hello-api.sh --artifacts-dir <tmp>`
- `scripts/check-runtime-smoke-artifacts.sh --artifacts-dir <tmp>`
