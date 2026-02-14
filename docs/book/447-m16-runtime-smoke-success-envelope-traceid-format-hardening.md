# 447 M16 Slice: Runtime-Smoke Success Envelope traceId Format Hardening

This chapter documents enforcing a deterministic `traceId` format in runtime-smoke success-envelope checks.

## 1) What it is

`scripts/check-runtime-smoke-artifacts.sh` now requires `users.body.traceId` to match:

- `^rt-[0-9]+$`

Regression coverage adds a malformed-trace fixture with `traceId="trace-1"` and asserts checker failure.

## 2) Why it exists

A non-empty traceId check is weaker than the runtime contract and allows drift in correlation-id shape. This slice aligns artifact validation with deterministic runtime trace format used by smoke evidence.

## 3) How it works internally

1. Success-envelope jq contract replaces generic non-empty trace check with regex-based check.
2. Test harness writes malformed traceId payload in a copied fixture.
3. Checker must fail with the same deterministic envelope-contract diagnostic.
4. Real smoke flow is re-run to prove runtime output still matches tightened contract.

## 4) Inputs, outputs, constraints

Inputs:

- `users.body` success-envelope artifact
- jq-based traceId format predicate

Outputs:

- deterministic pass/fail envelope validation based on traceId pattern

Constraints:

- contract currently assumes runtime trace IDs use `rt-<digits>` format
- future traceId format changes must update checker + tests + docs in one slice

## 5) Failure modes and diagnostics

- malformed traceId format:
  - checker exits non-zero with:
    - `users.body does not match expected std-success envelope contract`

## 6) Example usage

```bash
scripts/test-check-runtime-smoke-artifacts.sh
scripts/smoke-sec4-run-hello-api.sh --artifacts-dir /tmp/sec4-runtime-smoke
scripts/check-runtime-smoke-artifacts.sh --artifacts-dir /tmp/sec4-runtime-smoke
```

## 7) Trade-offs and next steps

Trade-offs:

- format-locked checks increase maintenance if traceId strategy evolves.

Next steps:

- add paired header/body trace correlation check (`X-Trace-Id == body.traceId`) in runtime-smoke checker for end-to-end correlation integrity.

## Verification

- `scripts/test-check-runtime-smoke-artifacts.sh`
- `scripts/smoke-sec4-run-hello-api.sh --artifacts-dir <tmp>`
- `scripts/check-runtime-smoke-artifacts.sh --artifacts-dir <tmp>`
