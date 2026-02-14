# 425 M16 Slice: Runtime HTTP Coverage Closure Gate

This chapter documents strict-closure enforcement for M16 runtime HTTP end-to-end coverage contracts.

## What it is

A new strict closure gate was added:

- `M16-A` in `scripts/check-milestone-closure.sh`

The gate requires naming-lock CI to execute both scripts:

- `scripts/test-m16-runtime-http-coverage.sh`
- `scripts/test-m16-runtime-http-coverage-guard.sh`

## Why it exists

M16 runtime behavior has broad HTTP security coverage (CORS, security headers, auth, CSRF, req.json gates). This gate makes sure that contract cannot silently drift by requiring an explicit coverage contract script and a negative guard test in CI.

## Implementation details

1. Added `scripts/test-m16-runtime-http-coverage.sh`:
   - verifies presence of required runtime e2e tests in `compiler/sec4-cli/tests/json_output.rs`.
2. Added `scripts/test-m16-runtime-http-coverage-guard.sh`:
   - validates checker pass/fail behavior with synthetic fixtures.
3. Wired both scripts into `.github/workflows/naming-lock.yml`.
4. Extended strict closure audit with `M16-A` and updated closure fixtures/docs.

## Validation

Executed locally:

- `scripts/test-m16-runtime-http-coverage.sh`
- `scripts/test-m16-runtime-http-coverage-guard.sh`
- `scripts/test-check-milestone-closure.sh`
- `scripts/test-roadmap-closure-gate-alignment.sh`
- `scripts/check-milestone-closure.sh --format json --fail-on-pending`

Result: pass.

## Tradeoffs and next steps

- This contract is intentionally explicit and function-name based; renaming tests requires deliberate contract updates.
- Next step can expand from presence checks to richer branch-metadata checks if M16 scope grows.
