# 167 M7 Slice: err.withPath Error-Argument Hardening

This chapter documents a focused M7 hardening step for `err.withPath` base-error typing.

## What it is

Extended `err.withPath` checks so:
- call shape remains `err.withPath(error, path)`,
- first argument (`error`) must be `StdError`.

Violations emit `E4001` with `security` tags.

## Why it exists

`err.withPath` is an error enricher. Requiring `StdError` as the base value prevents accidental chaining from non-error placeholders and keeps error-object composition deterministic.

## How it works internally

In `enforce_error_helper_signatures`:
1. detect `err.withPath` calls,
2. preserve existing arity and path-string checks,
3. validate argument 1 as `StdError`,
4. emit tagged diagnostics with canonical fix guidance.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/sec4-core/src/semantic.rs`
  - semantic fixture:
    - `invalid_err_with_path_error_argument_type.ut`
  - diagnostic-tag coverage:
    - `compiler/sec4-core/tests/diagnostic_tags.rs`
- Outputs:
  - compile-time rejection of non-`StdError` base arguments in `err.withPath`,
  - stable diagnostics for CI/editor surfaces.
- Constraint:
  - this slice does not change runtime lowering for `err.withPath`.

## Failure modes and diagnostics

Example:
- `err.withPath(1, "$.field")` ->
  - `E4001`: err.withPath error argument must be `StdError`.

## Example usage

```ut
fn attachPath(base: StdError) -> StdError {
  err.withPath(base, "$.field")
}
```

## Tradeoffs and next steps

- Tradeoff: bootstrap code that used numeric placeholders as base error values now fails semantic checks.
- Next:
  - harden base-error typing across remaining `err.with*` enrichers for full contract consistency.
