# 446 M16 Slice: Runtime-Smoke Success Envelope timeMs Contract Hardening

This chapter documents tightening runtime-smoke response validation to require `timeMs` in success envelopes.

## 1) What it is

`scripts/check-runtime-smoke-artifacts.sh` now requires `users.body` to contain:

- `ok == true`
- `status == 201`
- non-empty string `traceId`
- numeric `timeMs`
- `data` key present

Regression coverage adds a malformed-envelope fixture that omits `timeMs`.

## 2) Why it exists

The standard success envelope contract includes `timeMs`, but checker validation previously accepted payloads without it. This slice aligns runtime-smoke evidence checks with the documented envelope shape.

## 3) How it works internally

1. Checker jq expression adds:
   - `(.timeMs | type == "number")`
2. Regression harness rewrites `users.body` without `timeMs`.
3. Checker must fail with deterministic envelope-contract diagnostic.
4. Real smoke flow is re-run to confirm runtime outputs still satisfy hardened contract.

## 4) Inputs, outputs, constraints

Inputs:

- `users.body` artifact JSON payload
- checker jq contract expression

Outputs:

- deterministic pass/fail for success-envelope shape compliance

Constraints:

- checker diagnostic message remains stable for malformed envelope failures
- jq contract should stay compatible with runtime numeric `timeMs` output

## 5) Failure modes and diagnostics

- missing or non-numeric `timeMs` in `users.body`:
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

- stricter envelope checking can break quickly if envelope schema evolves, requiring synchronized checker/test updates.

Next steps:

- add optional metadata contract assertions (`meta` type/shape) when runtime-smoke scenarios begin exercising `res.okMeta` payloads.

## Verification

- `scripts/test-check-runtime-smoke-artifacts.sh`
- `scripts/smoke-sec4-run-hello-api.sh --artifacts-dir <tmp>`
- `scripts/check-runtime-smoke-artifacts.sh --artifacts-dir <tmp>`
