# 456 M16 Slice: Runtime-Smoke Users-Branch max-body Correlation Regression Coverage

This chapter documents adding explicit regression coverage for the users-branch max-body correlation check.

## 1) What it is

`scripts/test-check-runtime-smoke-artifacts.sh` now includes a dedicated fixture where:

- `maxBodyBytes=2048` in metadata
- `health.run.log` includes `--max-body-bytes 2048`
- `users.run.log` omits `--max-body-bytes 2048`

Expected checker failure:

- `users.run.log missing --max-body-bytes 2048 invocation token`

## 2) Why it exists

Existing max-body mismatch regression covered the health branch first. This slice ensures the users branch is also contract-pinned and cannot regress silently.

## 3) How it works internally

1. Fixture copies the passing baseline artifact set.
2. Metadata is set to numeric max-body mode (`2048`).
3. Health log is patched to satisfy max-body correlation.
4. Users log intentionally stays without max-body flag.
5. Checker must reach users branch and emit deterministic mismatch diagnostic.

## 4) Inputs, outputs, constraints

Inputs:

- checker regression fixture artifacts

Outputs:

- deterministic users-branch mismatch assertion

Constraints:

- health branch must pass first; otherwise users-branch assertion is not reached
- diagnostic text is contract-locked

## 5) Failure modes and diagnostics

- users branch max-body mismatch:
  - `users.run.log missing --max-body-bytes 2048 invocation token`

## 6) Example usage

```bash
scripts/test-check-runtime-smoke-artifacts.sh
```

## 7) Trade-offs and next steps

Trade-offs:

- fixture matrix grows as branch-specific diagnostics are pinned.

Next steps:

- add analogous branch-specific coverage for timeout and port correlation checks if deeper branch granularity is needed.

## Verification

- `scripts/test-check-runtime-smoke-artifacts.sh`
- `scripts/check-runtime-smoke-artifacts.sh --artifacts-dir <tmp>`
