# 166 M7 Slice: err.withCause Argument Hardening

This chapter documents a focused M7 hardening step for `err.withCause`.

## What it is

Added semantic checks for `err.withCause`:
- call shape must be `err.withCause(error, cause)`,
- both arguments must be `StdError`.

Violations emit `E4001` with `security` tags.

## Why it exists

`err.withCause` should preserve the structured `StdError` chain. Allowing non-error payloads weakens error modeling and breaks deterministic tooling around error causes.

## How it works internally

In `enforce_error_helper_signatures`:
1. detect `err.withCause` calls,
2. enforce exact two-argument shape,
3. validate both argument types as `StdError`,
4. emit tagged diagnostics when contracts are violated.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/ailang-core/src/semantic.rs`
  - semantic fixtures:
    - `invalid_err_with_cause_error_argument_type.ai`
    - `invalid_err_with_cause_cause_argument_type.ai`
  - diagnostic-tag coverage:
    - `compiler/ailang-core/tests/diagnostic_tags.rs`
- Outputs:
  - compile-time rejection of invalid `err.withCause` arguments,
  - tagged diagnostics suitable for CLI/editor feedback.
- Constraint:
  - this slice hardens `err.withCause` only; other `err.with*` helpers keep their existing contracts.

## Failure modes and diagnostics

Examples:
- `err.withCause(1, cause)` ->
  - `E4001`: err.withCause error argument must be `StdError`.
- `err.withCause(base, 1)` ->
  - `E4001`: err.withCause cause argument must be `StdError`.

## Example usage

```ailang
fn buildError() -> StdError {
  let base = err.validation("VALIDATION.BAD", "invalid input");
  let cause = err.internal("db timeout");
  err.withCause(base, cause)
}
```

## Tradeoffs and next steps

- Tradeoff: placeholder bootstrap calls that used numeric arguments for cause chaining now fail semantic checks.
- Next:
  - extend `err.with*` helper hardening to validate base-error argument types consistently across all enrichers.
