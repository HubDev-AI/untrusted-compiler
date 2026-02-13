# 157 M7 Slice: err.validation Constructor-Argument Hardening

This chapter documents a focused M7 hardening step for typed validation-error constructor usage.

## What it is

Added semantic checks for `err.validation`:
- call shape must be `err.validation(code, message)`,
- `code` must be `String`,
- `message` must be `String`.

Violations emit `E4001` with `security` tags.

## Why it exists

Validation errors are part of the stable error contract. Allowing placeholder numeric values for error code/message weakens consistency and makes downstream error handling brittle.

## How it works internally

In semantic call enforcement:
1. detect `err.validation` calls,
2. enforce exact two-argument shape,
3. enforce string code/message types,
4. emit deterministic diagnostics with usage guidance.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/ailang-core/src/semantic.rs`
  - semantic fixture:
    - `invalid_err_validation_argument_type.ai`
  - diagnostic-tag coverage:
    - `compiler/ailang-core/tests/diagnostic_tags.rs`
  - integration fixture alignment:
    - `compiler/ailang-core/tests/c_backend.rs`
    - `compiler/ailang-cli/tests/json_output.rs`
- Outputs:
  - compile-time rejection of malformed validation-error constructor values,
  - stable diagnostics for tooling and CI.
- Constraint:
  - runtime error ABI behavior remains unchanged.

## Failure modes and diagnostics

Example:
- `err.validation(1, 2)` ->
  - `E4001`: code/message arguments must be `String`.

## Example usage

```ailang
fn makeValidation() -> Int {
  err.validation("VALIDATION.BAD_REQUEST", "invalid input");
  0
}
```

## Tradeoffs and next steps

- Tradeoff: permissive constructor placeholders from early bridge fixtures now fail semantic checks.
- Next:
  - continue constructor typing hardening for `err.auth`, `err.notFound`, `err.conflict`, and `err.rateLimit`.
