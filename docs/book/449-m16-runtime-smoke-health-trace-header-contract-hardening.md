# 449 M16 Slice: Runtime-Smoke Health Trace-Header Contract Hardening

This chapter documents extending runtime-smoke trace validation to the `/health` response header contract.

## 1) What it is

`scripts/check-runtime-smoke-artifacts.sh` now requires `health.headers` to include:

- `X-Trace-Id`
- value format `rt-[0-9]+`

Regression coverage adds fixtures for missing and malformed health trace headers.

## 2) Why it exists

Trace-correlation hardening covered users responses, but `/health` remained unchecked for trace-header presence/shape. Adding this check keeps runtime-smoke trace contracts consistent across both exercised endpoints.

## 3) How it works internally

1. Checker validates health status line.
2. Checker enforces `X-Trace-Id` presence in `health.headers`.
3. Checker extracts value and validates regex `^rt-[0-9]+$`.
4. Regression harness exercises missing-header and malformed-header failures with deterministic diagnostics.

## 4) Inputs, outputs, constraints

Inputs:

- `health.headers` artifact file

Outputs:

- deterministic pass/fail diagnostics for health trace-header contract

Constraints:

- trace format check assumes runtime emits `rt-<digits>` identifiers
- health response is plain text, so this slice validates only header-level trace contract

## 5) Failure modes and diagnostics

- missing health trace header:
  - `health.headers missing X-Trace-Id header`
- malformed health trace value:
  - `health.headers contains malformed X-Trace-Id value`

## 6) Example usage

```bash
scripts/test-check-runtime-smoke-artifacts.sh
scripts/smoke-sec4-run-hello-api.sh --artifacts-dir /tmp/sec4-runtime-smoke
scripts/check-runtime-smoke-artifacts.sh --artifacts-dir /tmp/sec4-runtime-smoke
```

## 7) Trade-offs and next steps

Trade-offs:

- checker contract grows stricter; runtime header-format changes now require synchronized checker/test/doc updates.

Next steps:

- add cross-endpoint trace uniqueness checks if runtime-smoke flow starts covering multi-request single-process runs.

## Verification

- `scripts/test-check-runtime-smoke-artifacts.sh`
- `scripts/smoke-sec4-run-hello-api.sh --artifacts-dir <tmp>`
- `scripts/check-runtime-smoke-artifacts.sh --artifacts-dir <tmp>`
