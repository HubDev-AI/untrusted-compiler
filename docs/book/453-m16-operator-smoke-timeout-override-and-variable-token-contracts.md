# 453 M16 Slice: Operator Smoke Timeout Override and Variable Token Contracts

This chapter documents adding timeout override support to the operator smoke script and locking variable-based timeout tokens in contract checks.

## 1) What it is

`scripts/smoke-sec4-run-hello-api.sh` now accepts:

- `--serve-timeout-ms <ms>`

with default:

- `serve_timeout_ms="12000"`

The override value is propagated to:

- `sec4 run ... --serve-timeout-ms "<value>"`
- `run-metadata.txt` as `serveTimeoutMs=<value>`

Smoke-script contract tests now enforce variable-based tokens instead of fixed literals.

## 2) Why it exists

Operator smoke runs previously hardcoded timeout `12000`, limiting test flexibility. This slice keeps deterministic defaults while allowing controlled timeout overrides without breaking contract coverage.

## 3) How it works internally

1. Script argument parser adds `--serve-timeout-ms` flag.
2. Input validation enforces positive integer timeout values.
3. Invocation path and metadata emitter both reference `serve_timeout_ms`.
4. Contract checker required-token list now locks:
   - default variable assignment
   - invocation token using `${serve_timeout_ms}`
   - metadata token using `${serve_timeout_ms}`
5. Guard test adds timeout-token removal scenarios for both invocation and metadata lines.

## 4) Inputs, outputs, constraints

Inputs:

- optional timeout override flag from operator

Outputs:

- runtime smoke invocation with selected timeout
- metadata and run-log artifacts that remain checker-compatible

Constraints:

- timeout must be positive integer
- runFlags metadata remains name-only contract (`--port,--oneshot,--serve-timeout-ms`)

## 5) Failure modes and diagnostics

- invalid timeout input:
  - `invalid --serve-timeout-ms value (expected positive integer): <value>`
- missing timeout invocation/metadata tokens:
  - contract guard fails with deterministic missing-token diagnostics

## 6) Example usage

```bash
scripts/smoke-sec4-run-hello-api.sh --artifacts-dir /tmp/sec4-runtime-smoke
scripts/smoke-sec4-run-hello-api.sh --serve-timeout-ms 9000 --artifacts-dir /tmp/sec4-runtime-smoke-9000
scripts/check-runtime-smoke-artifacts.sh --artifacts-dir /tmp/sec4-runtime-smoke-9000
```

## 7) Trade-offs and next steps

Trade-offs:

- contract checks become slightly more implementation-aware by locking variable-token text.

Next steps:

- add optional `--max-body-bytes` passthrough to smoke script if operator runtime-smoke scenarios need parameterized size-limit coverage.

## Verification

- `scripts/test-smoke-sec4-run-hello-api-script-contract.sh`
- `scripts/test-smoke-sec4-run-hello-api-script-contract-guard.sh`
- `scripts/test-check-runtime-smoke-artifacts.sh`
- `scripts/smoke-sec4-run-hello-api.sh --artifacts-dir <tmp>`
- `scripts/smoke-sec4-run-hello-api.sh --serve-timeout-ms 9000 --artifacts-dir <tmp>`
- `scripts/check-runtime-smoke-artifacts.sh --artifacts-dir <tmp>`
