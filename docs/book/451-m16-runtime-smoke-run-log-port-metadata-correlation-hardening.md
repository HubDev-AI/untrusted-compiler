# 451 M16 Slice: Runtime-Smoke Run-Log Port/Metadata Correlation Hardening

This chapter documents enforcing exact `--port` correlation between runtime-smoke run logs and `run-metadata.txt`.

## 1) What it is

`scripts/check-runtime-smoke-artifacts.sh` now parses `port=<N>` from metadata and requires both run logs to include:

- `--port <N>`

Regression coverage includes a fixture where metadata port and run-log port diverge.

## 2) Why it exists

Checking only generic `--port` token presence does not prove logs correspond to the recorded metadata. This slice enforces value-level consistency and closes that provenance gap.

## 3) How it works internally

1. Checker validates numeric metadata port.
2. Checker extracts `port_value` from metadata.
3. Checker enforces exact `--port <port_value>` token in:
   - `health.run.log`
   - `users.run.log`
4. Regression fixture sets metadata port to `9090` while logs keep `8080`, and asserts deterministic failure.

## 4) Inputs, outputs, constraints

Inputs:

- metadata `port` field
- health/users run-log files

Outputs:

- deterministic correlation pass/fail diagnostics

Constraints:

- checker uses token matching, not full command-line parsing
- first metadata `port` value is treated as canonical

## 5) Failure modes and diagnostics

- metadata/log port mismatch:
  - `health.run.log missing --port <N> invocation token`
  (fails first on health log; users log check runs only if health passes)

## 6) Example usage

```bash
scripts/test-check-runtime-smoke-artifacts.sh
scripts/smoke-sec4-run-hello-api.sh --artifacts-dir /tmp/sec4-runtime-smoke
scripts/check-runtime-smoke-artifacts.sh --artifacts-dir /tmp/sec4-runtime-smoke
```

## 7) Trade-offs and next steps

Trade-offs:

- contract now couples run-log checks to metadata value extraction order.

Next steps:

- add analogous strict correlation for timeout value (`serveTimeoutMs`) if smoke scenarios begin varying timeout settings.

## Verification

- `scripts/test-check-runtime-smoke-artifacts.sh`
- `scripts/smoke-sec4-run-hello-api.sh --artifacts-dir <tmp>`
- `scripts/check-runtime-smoke-artifacts.sh --artifacts-dir <tmp>`
