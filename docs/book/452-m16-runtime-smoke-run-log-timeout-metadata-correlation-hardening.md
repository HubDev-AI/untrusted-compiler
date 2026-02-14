# 452 M16 Slice: Runtime-Smoke Run-Log Timeout/Metadata Correlation Hardening

This chapter documents enforcing exact timeout-token correlation between runtime-smoke run logs and metadata.

## 1) What it is

`scripts/check-runtime-smoke-artifacts.sh` now parses `serveTimeoutMs=<N>` from metadata and requires both run logs to include:

- `--serve-timeout-ms <N>`

Regression coverage includes a fixture where metadata timeout and run-log timeout diverge.

## 2) Why it exists

Checking a fixed timeout token proves one static scenario, but not value consistency with recorded metadata. This slice hardens provenance by correlating run-log timeout tokens with metadata timeout.

## 3) How it works internally

1. Checker validates numeric `serveTimeoutMs` metadata.
2. Checker extracts `serve_timeout_ms` from metadata.
3. Checker enforces exact timeout token in health/users run logs.
4. Regression fixture sets metadata timeout to `9000` while logs remain `12000`; checker must fail deterministically.

## 4) Inputs, outputs, constraints

Inputs:

- metadata timeout field
- health/users run logs

Outputs:

- deterministic timeout-correlation pass/fail diagnostics

Constraints:

- checker uses token matching against captured command lines
- first metadata timeout value is treated as canonical

## 5) Failure modes and diagnostics

- metadata/log timeout mismatch:
  - `health.run.log missing --serve-timeout-ms <N> invocation token`

## 6) Example usage

```bash
scripts/test-check-runtime-smoke-artifacts.sh
scripts/smoke-sec4-run-hello-api.sh --artifacts-dir /tmp/sec4-runtime-smoke
scripts/check-runtime-smoke-artifacts.sh --artifacts-dir /tmp/sec4-runtime-smoke
```

## 7) Trade-offs and next steps

Trade-offs:

- correlation checks increase coupling between metadata schema and run-log token assertions.

Next steps:

- consider deriving `runFlags` token order from parsed invocation lines to lock metadata/log parity even tighter.

## Verification

- `scripts/test-check-runtime-smoke-artifacts.sh`
- `scripts/smoke-sec4-run-hello-api.sh --artifacts-dir <tmp>`
- `scripts/check-runtime-smoke-artifacts.sh --artifacts-dir <tmp>`
