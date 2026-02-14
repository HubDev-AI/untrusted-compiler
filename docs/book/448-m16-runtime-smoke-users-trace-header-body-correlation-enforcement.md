# 448 M16 Slice: Runtime-Smoke Users Trace Header/Body Correlation Enforcement

This chapter documents runtime-smoke validation hardening for users-response trace correlation across headers and body.

## 1) What it is

`scripts/check-runtime-smoke-artifacts.sh` now enforces:

- `users.headers` must contain `X-Trace-Id`
- `X-Trace-Id` value must equal `users.body.traceId`

Regression coverage adds fixtures for missing users trace header and header/body mismatch.

## 2) Why it exists

Success-envelope checks validated `traceId` inside body, but did not verify header propagation and consistency. This slice locks end-to-end trace correlation integrity for runtime-smoke evidence.

## 3) How it works internally

1. Checker validates success-envelope body contract (existing).
2. Checker verifies `users.headers` has a non-empty `X-Trace-Id` line.
3. Checker extracts header trace and compares with `users.body.traceId`.
4. Checker fails with deterministic diagnostics for missing header or mismatch.
5. Regression harness includes both negative scenarios.

## 4) Inputs, outputs, constraints

Inputs:

- users response headers artifact
- users response body artifact

Outputs:

- deterministic trace-correlation pass/fail diagnostics

Constraints:

- header parsing assumes canonical `X-Trace-Id` format and first-match usage
- diagnostic strings are contract-locked by tests

## 5) Failure modes and diagnostics

- missing users trace header:
  - `users.headers missing X-Trace-Id header`
- header/body trace mismatch:
  - `users traceId mismatch between headers and body`

## 6) Example usage

```bash
scripts/test-check-runtime-smoke-artifacts.sh
scripts/smoke-sec4-run-hello-api.sh --artifacts-dir /tmp/sec4-runtime-smoke
scripts/check-runtime-smoke-artifacts.sh --artifacts-dir /tmp/sec4-runtime-smoke
```

## 7) Trade-offs and next steps

Trade-offs:

- checker now parses and compares cross-file values, increasing contract strictness and maintenance burden.

Next steps:

- add analogous trace correlation check for `/health` response headers to cover both runtime-smoke requests consistently.

## Verification

- `scripts/test-check-runtime-smoke-artifacts.sh`
- `scripts/smoke-sec4-run-hello-api.sh --artifacts-dir <tmp>`
- `scripts/check-runtime-smoke-artifacts.sh --artifacts-dir <tmp>`
