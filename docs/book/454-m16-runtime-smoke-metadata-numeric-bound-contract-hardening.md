# 454 M16 Slice: Runtime-Smoke Metadata Numeric-Bound Contract Hardening

This chapter documents adding numeric-bound checks for runtime-smoke metadata fields.

## 1) What it is

`scripts/check-runtime-smoke-artifacts.sh` now enforces:

- `port` must be between `1` and `65535`
- `serveTimeoutMs` must be greater than `0`

Regression coverage includes dedicated fixtures for invalid `port=0` and `serveTimeoutMs=0`.

## 2) Why it exists

Previous checks validated numeric shape only (`[0-9]+`) and allowed semantically invalid values. This slice hardens metadata contracts so checker validations better reflect runtime-meaningful bounds.

## 3) How it works internally

1. Checker parses `port` and `serveTimeoutMs` from metadata.
2. Checker enforces explicit range/bound rules.
3. Regression harness injects invalid values and asserts deterministic failures.
4. Default and timeout-override smoke flows are revalidated to ensure compatibility.

## 4) Inputs, outputs, constraints

Inputs:

- metadata numeric fields in `run-metadata.txt`

Outputs:

- deterministic bound-validation diagnostics

Constraints:

- checks assume decimal integer metadata values
- range check order influences first-reported failure diagnostic

## 5) Failure modes and diagnostics

- invalid port range:
  - `run-metadata.txt port must be between 1 and 65535`
- invalid timeout bound:
  - `run-metadata.txt serveTimeoutMs must be greater than 0`

## 6) Example usage

```bash
scripts/test-check-runtime-smoke-artifacts.sh
scripts/smoke-sec4-run-hello-api.sh --artifacts-dir /tmp/sec4-runtime-smoke
scripts/smoke-sec4-run-hello-api.sh --serve-timeout-ms 9000 --artifacts-dir /tmp/sec4-runtime-smoke-9000
scripts/check-runtime-smoke-artifacts.sh --artifacts-dir /tmp/sec4-runtime-smoke-9000
```

## 7) Trade-offs and next steps

Trade-offs:

- checker now carries semantic numeric assumptions that must stay aligned with runtime CLI bounds.

Next steps:

- align checker numeric-bound diagnostics with `sec4 run` CLI validation messages to reduce operator confusion when bounds fail.

## Verification

- `scripts/test-check-runtime-smoke-artifacts.sh`
- `scripts/smoke-sec4-run-hello-api.sh --artifacts-dir <tmp>`
- `scripts/smoke-sec4-run-hello-api.sh --serve-timeout-ms 9000 --artifacts-dir <tmp>`
- `scripts/check-runtime-smoke-artifacts.sh --artifacts-dir <tmp>`
