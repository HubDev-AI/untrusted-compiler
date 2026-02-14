# 437 M16 Slice: Smoke-Script Metadata Token Contract Lock

This chapter documents locking deterministic metadata emit tokens directly in the smoke-script contract checker.

## 1) What it is

`scripts/test-smoke-sec4-run-hello-api-script-contract.sh` now requires smoke-script source tokens for:

- `oneshot=true`
- `serveTimeoutMs=12000`
- `runFlags=--port,--oneshot,--serve-timeout-ms`

## 2) Why it exists

`M16-S32` expanded artifact metadata and checker validation, but metadata emit drift in the smoke script could still pass unnoticed until checker/runtime execution. Token-level contract checks catch that drift earlier at script-contract stage.

## 3) How it works internally

1. Added metadata-field tokens to `required_tokens` in smoke-script contract test.
2. Existing guard test continues validating contract-failure behavior on token removal.
3. Real smoke + artifact checker flow remains the end-to-end behavior validation layer.

## 4) Inputs, outputs, constraints

Inputs:

- smoke-script source file path (default `scripts/smoke-sec4-run-hello-api.sh`)

Outputs:

- deterministic pass/fail contract status for metadata token presence

Constraints:

- token checks are structural and complement (not replace) runtime artifact checks.

## 5) Failure modes and diagnostics

- missing metadata token causes: `missing required smoke-script token: <token>`
- guard test fails if expected contract-failure path no longer triggers

## 6) Example usage

```bash
scripts/test-smoke-sec4-run-hello-api-script-contract.sh
scripts/test-smoke-sec4-run-hello-api-script-contract-guard.sh
```

## 7) Trade-offs and next steps

Trade-offs:

- source token contracts can be brittle if script structure/text changes.

Next steps:

- if smoke script evolves into helper functions/templates, consider moving token checks to a small parsed metadata-emission contract helper to reduce brittleness.

## Verification

- `scripts/test-smoke-sec4-run-hello-api-script-contract.sh`
- `scripts/test-smoke-sec4-run-hello-api-script-contract-guard.sh`
- `scripts/smoke-sec4-run-hello-api.sh --artifacts-dir <tmp>`
- `scripts/check-runtime-smoke-artifacts.sh --artifacts-dir <tmp>`
