# 170 M7 Slice: err.withDetail Error-Argument Hardening

This chapter documents a focused M7 hardening step for `err.withDetail` base-error typing.

## What it is

Extended `err.withDetail` checks so:
- call shape remains `err.withDetail(error, key, value)`,
- first argument (`error`) must be `StdError`.

Violations emit `E4001` with `security` tags.

## Why it exists

`err.withDetail` enriches structured errors. Requiring a canonical `StdError` base prevents accidental detail attachment to non-error placeholders and keeps error chains consistent.

## How it works internally

In `enforce_error_helper_signatures`:
1. detect `err.withDetail` calls,
2. preserve key/value safety checks,
3. validate argument 1 as `StdError`,
4. emit deterministic tagged diagnostics when invalid.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/sec4-core/src/semantic.rs`
  - semantic fixture:
    - `invalid_err_with_detail_error_argument_type.ut`
  - diagnostic-tag coverage:
    - `compiler/sec4-core/tests/diagnostic_tags.rs`
- Outputs:
  - compile-time rejection of non-`StdError` `err.withDetail` base arguments,
  - stable diagnostics for tooling and CI.
- Constraint:
  - runtime helper lowering remains unchanged.

## Failure modes and diagnostics

Example:
- `err.withDetail(1, "token", 2)` ->
  - `E4001`: err.withDetail error argument must be `StdError`.

## Example usage

```ut
fn attachDetail(base: StdError) -> StdError {
  err.withDetail(base, "field", 2)
}
```

## Tradeoffs and next steps

- Tradeoff: bootstrap fixtures that used numeric base placeholders now fail semantic checks.
- Next:
  - continue applying strict `StdError` base contracts to any remaining error enrichers.
