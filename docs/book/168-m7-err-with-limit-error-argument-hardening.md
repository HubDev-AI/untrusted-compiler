# 168 M7 Slice: err.withLimit Error-Argument Hardening

This chapter documents a focused M7 hardening step for `err.withLimit` base-error typing.

## What it is

Extended `err.withLimit` checks so:
- call shape remains `err.withLimit(error, name, max, actual)`,
- first argument (`error`) must be `StdError`.

Violations emit `E4001` with `security` tags.

## Why it exists

`err.withLimit` enriches error objects with limit metadata. Enforcing a real `StdError` base value prevents accidental misuse with placeholder numerics and keeps error composition consistent.

## How it works internally

In `enforce_error_helper_signatures`:
1. detect `err.withLimit` calls,
2. preserve existing shape/name/max/actual checks,
3. validate argument 1 as `StdError`,
4. emit deterministic tagged diagnostics when invalid.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/sec4-core/src/semantic.rs`
  - semantic fixture:
    - `invalid_err_with_limit_error_argument_type.ut`
  - diagnostic-tag coverage:
    - `compiler/sec4-core/tests/diagnostic_tags.rs`
- Outputs:
  - compile-time rejection of non-`StdError` `err.withLimit` base arguments,
  - stable diagnostics for CI/editor flows.
- Constraint:
  - runtime error-helper lowering remains unchanged.

## Failure modes and diagnostics

Example:
- `err.withLimit(1, "limit", 10, 11)` ->
  - `E4001`: err.withLimit error argument must be `StdError`.

## Example usage

```ut
fn attachLimit(base: StdError) -> StdError {
  err.withLimit(base, "json.max_depth", 32, 40)
}
```

## Tradeoffs and next steps

- Tradeoff: bridge-stage samples that used numeric base placeholders now fail semantic checks.
- Next:
  - apply the same base-error typing requirement to `err.withDependency` and `err.withDetail`.
