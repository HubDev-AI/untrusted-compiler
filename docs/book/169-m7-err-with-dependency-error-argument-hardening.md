# 169 M7 Slice: err.withDependency Error-Argument Hardening

This chapter documents a focused M7 hardening step for `err.withDependency` base-error typing.

## What it is

Extended `err.withDependency` checks so:
- call shape remains `err.withDependency(error, name, operation, retryable)`,
- first argument (`error`) must be `StdError`.

Violations emit `E4001` with `security` tags.

## Why it exists

`err.withDependency` enriches structured errors. Enforcing `StdError` as the base value ensures dependency metadata is attached only to canonical error objects.

## How it works internally

In `enforce_error_helper_signatures`:
1. detect `err.withDependency` calls,
2. preserve existing name/operation/retryable checks,
3. validate argument 1 as `StdError`,
4. emit deterministic tagged diagnostics when invalid.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/sec4-core/src/semantic.rs`
  - semantic fixture:
    - `invalid_err_with_dependency_error_argument_type.ut`
  - diagnostic-tag coverage:
    - `compiler/sec4-core/tests/diagnostic_tags.rs`
- Outputs:
  - compile-time rejection of non-`StdError` `err.withDependency` base arguments,
  - stable diagnostics for tooling and CI.
- Constraint:
  - runtime bridge lowering for `err.withDependency` is unchanged.

## Failure modes and diagnostics

Example:
- `err.withDependency(1, "dep", "op", true)` ->
  - `E4001`: err.withDependency error argument must be `StdError`.

## Example usage

```ut
fn attachDependency(base: StdError) -> StdError {
  err.withDependency(base, "postgres", "query", true)
}
```

## Tradeoffs and next steps

- Tradeoff: bootstrap samples that used numeric base placeholders now fail semantic checks.
- Next:
  - apply consistent `StdError` base typing to remaining `err.with*` enrichers (`err.withDetail`).
