# 455 M16 Slice: Operator Smoke max-body Override and maxBody Metadata/Log Contracts

This chapter documents adding max-body override support to the operator smoke script and hardening checker contracts around `maxBodyBytes`.

## 1) What it is

`scripts/smoke-sec4-run-hello-api.sh` now accepts:

- `--max-body-bytes <bytes>`

When provided, the script:

- appends `--max-body-bytes <bytes>` to `sec4 run`
- emits `maxBodyBytes=<bytes>` in metadata

When omitted, metadata emits:

- `maxBodyBytes=unset`

Checker contracts now validate `maxBodyBytes` shape, bounds, and optional run-log correlation.

## 2) Why it exists

Timeout override was already supported, but body-size override was missing from operator smoke flows. This slice enables controlled body-limit runs and keeps artifact contracts strict and auditable.

## 3) How it works internally

1. Smoke script parser adds optional max-body flag with positive-integer validation.
2. Invocation builder conditionally appends max-body flag.
3. Metadata writer emits `maxBodyBytes` in all runs (`unset` or numeric).
4. Contract checker required tokens are updated for:
   - `max_body_bytes` variable
   - max-body invocation token
   - max-body metadata token
5. Guard tests add removal scenarios for max-body invocation/metadata tokens.
6. Runtime-smoke checker now enforces:
   - `maxBodyBytes` field presence/shape (`unset|[0-9]+`)
   - positive numeric bound when set
   - exact `--max-body-bytes <value>` token in run logs when set

## 4) Inputs, outputs, constraints

Inputs:

- optional `--max-body-bytes` operator flag

Outputs:

- metadata with explicit max-body mode (`unset` or numeric)
- run logs correlated with max-body setting when numeric

Constraints:

- numeric max-body value must be strictly positive
- runFlags metadata remains name-only contract and does not embed max-body value

## 5) Failure modes and diagnostics

- invalid max-body input in smoke script:
  - `invalid --max-body-bytes value (expected positive integer): <value>`
- checker missing/invalid metadata:
  - `run-metadata.txt missing or invalid maxBodyBytes field`
- checker non-positive metadata value:
  - `run-metadata.txt maxBodyBytes must be unset or positive integer`
- checker metadata/log mismatch:
  - `health.run.log missing --max-body-bytes <value> invocation token`

## 6) Example usage

```bash
scripts/smoke-sec4-run-hello-api.sh --artifacts-dir /tmp/sec4-runtime-smoke
scripts/smoke-sec4-run-hello-api.sh --max-body-bytes 2048 --artifacts-dir /tmp/sec4-runtime-smoke-maxbody
scripts/check-runtime-smoke-artifacts.sh --artifacts-dir /tmp/sec4-runtime-smoke-maxbody
```

## 7) Trade-offs and next steps

Trade-offs:

- checker and contract tests now track additional optional-mode behavior, increasing fixture complexity.

Next steps:

- consider adding `runFlagsDetailed` metadata with value-bearing optional flags if operator replay/debug tooling needs explicit mode reconstruction.

## Verification

- `scripts/test-smoke-sec4-run-hello-api-script-contract.sh`
- `scripts/test-smoke-sec4-run-hello-api-script-contract-guard.sh`
- `scripts/test-check-runtime-smoke-artifacts.sh`
- `scripts/smoke-sec4-run-hello-api.sh --artifacts-dir <tmp>`
- `scripts/smoke-sec4-run-hello-api.sh --serve-timeout-ms 9000 --artifacts-dir <tmp>`
- `scripts/smoke-sec4-run-hello-api.sh --max-body-bytes 2048 --artifacts-dir <tmp>`
- `scripts/check-runtime-smoke-artifacts.sh --artifacts-dir <tmp>`
