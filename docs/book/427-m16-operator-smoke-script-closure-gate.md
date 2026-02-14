# 427 M16 Slice: Operator Smoke Script Closure Gate

This chapter documents strict-closure enforcement for the operator smoke script contract added in M16-S25.

## What it is

A new strict closure gate was added:

- `M16-B` in `scripts/check-milestone-closure.sh`

The gate requires naming-lock CI to execute:

- `scripts/test-smoke-sec4-run-hello-api-script-contract.sh`
- `scripts/test-smoke-sec4-run-hello-api-script-contract-guard.sh`

## Why it exists

The smoke script is an operator reliability primitive. Without closure wiring, it could silently drift (for example, dropping auth/CSRF checks) while still existing in the repo. `M16-B` makes its essential behavior auditable and non-regressible.

## Implementation details

1. Added smoke-script contract checker:
   - validates executable presence,
   - validates required runtime/auth/csrf/assertion tokens.
2. Added guard checker:
   - validates checker failure when required contract token is removed.
3. Wired both checks into naming-lock CI.
4. Expanded strict closure audit with `M16-B` and updated fixture/docs alignment.

## Validation

Executed locally:

- `scripts/test-smoke-sec4-run-hello-api-script-contract.sh`
- `scripts/test-smoke-sec4-run-hello-api-script-contract-guard.sh`
- `scripts/test-check-milestone-closure.sh`
- `scripts/test-roadmap-closure-gate-alignment.sh`
- `scripts/check-milestone-closure.sh --format json --fail-on-pending`

Result: pass.

## Tradeoffs and next steps

- This contract intentionally checks stable behavioral tokens instead of parsing shell AST to keep guard execution simple and deterministic.
- Future hardening can add optional semantic checks for request headers/path validations if script complexity grows.
