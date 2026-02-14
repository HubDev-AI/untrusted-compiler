# 450 M16 Slice: Runtime-Smoke Run-Log Invocation Flag Contract Hardening

This chapter documents enforcing invocation-flag evidence in runtime-smoke run-log artifacts.

## 1) What it is

`scripts/check-runtime-smoke-artifacts.sh` now requires both `health.run.log` and `users.run.log` to include:

- `--port`
- `--oneshot`
- `--serve-timeout-ms 12000`

Regression fixtures now cover missing-token failures for both logs.

## 2) Why it exists

Static smoke-script contract checks prove script text, but runtime artifact checks should also prove executed invocation semantics. This slice ensures run logs carry the expected runtime flags.

## 3) How it works internally

1. Checker validates run-log token presence for health and users invocations.
2. Regression harness uses copied fixtures:
   - health log without `--oneshot`
   - users log without `--serve-timeout-ms 12000`
3. Checker must fail with deterministic missing-token diagnostics.
4. Real smoke flow is re-run to confirm runtime logs satisfy hardened contract.

## 4) Inputs, outputs, constraints

Inputs:

- `health.run.log`
- `users.run.log`

Outputs:

- deterministic run-log contract pass/fail diagnostics

Constraints:

- checker relies on token presence, not full command-line parsing
- timeout token is currently fixed to `12000` for smoke-contract stability

## 5) Failure modes and diagnostics

- missing health oneshot token:
  - `health.run.log missing --oneshot invocation token`
- missing users timeout token:
  - `users.run.log missing --serve-timeout-ms 12000 invocation token`

## 6) Example usage

```bash
scripts/test-check-runtime-smoke-artifacts.sh
scripts/smoke-sec4-run-hello-api.sh --artifacts-dir /tmp/sec4-runtime-smoke
scripts/check-runtime-smoke-artifacts.sh --artifacts-dir /tmp/sec4-runtime-smoke
```

## 7) Trade-offs and next steps

Trade-offs:

- checker encodes fixed-token assumptions tied to smoke-script flag defaults.

Next steps:

- add structured parsing of invocation lines if smoke scenarios start varying timeout values per run.

## Verification

- `scripts/test-check-runtime-smoke-artifacts.sh`
- `scripts/smoke-sec4-run-hello-api.sh --artifacts-dir <tmp>`
- `scripts/check-runtime-smoke-artifacts.sh --artifacts-dir <tmp>`
