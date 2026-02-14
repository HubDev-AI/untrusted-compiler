# 438 M16 Slice: Smoke-Script Metadata Guard Coverage Expansion

This chapter documents extending smoke-script contract guard coverage to include metadata token drift scenarios.

## 1) What it is

`scripts/test-smoke-sec4-run-hello-api-script-contract-guard.sh` now checks two failure classes:

- request token drift (`X-CSRF-Token: token123` removal)
- metadata token drift (`runFlags=--port,--oneshot,--serve-timeout-ms` removal)

## 2) Why it exists

After adding metadata token requirements in smoke-script contract checks, guard coverage still validated only request-token removal. That left metadata guard behavior unproven.

This slice adds explicit metadata guard assertions to keep failure behavior deterministic across both token categories.

## 3) How it works internally

1. Existing CSRF-token mutation scenario remains unchanged.
2. Added a second fixture copy (`smoke-meta.sh`) and mutates `runFlags=...` token.
3. Contract script is expected to fail, and guard asserts exact missing-token diagnostic for metadata token.

## 4) Inputs, outputs, constraints

Inputs:

- smoke script fixture copy for request-token mutation
- smoke script fixture copy for metadata-token mutation

Outputs:

- deterministic pass/fail guard status across both drift scenarios

Constraints:

- guard remains token-driven and depends on stable contract diagnostic wording.

## 5) Failure modes and diagnostics

New enforced diagnostic path:

- `missing required smoke-script token: runFlags=--port,--oneshot,--serve-timeout-ms`

## 6) Example usage

```bash
scripts/test-smoke-sec4-run-hello-api-script-contract-guard.sh
```

## 7) Trade-offs and next steps

Trade-offs:

- additional guard scenario increases script complexity slightly.

Next steps:

- if contract script expands further, consider moving guard scenarios to table-driven helper function to keep maintenance simple.

## Verification

- `scripts/test-smoke-sec4-run-hello-api-script-contract.sh`
- `scripts/test-smoke-sec4-run-hello-api-script-contract-guard.sh`
