# 156 M7 Slice: err.internal Message-Argument Hardening

This chapter documents a focused M7 hardening step for typed internal-error constructor usage.

## What it is

Added semantic checks for `err.internal`:
- call shape must be `err.internal(message)`,
- `message` must be `String`.

Violations emit `E4001` with `security` tags.

## Why it exists

`err.internal` is the baseline internal-error constructor. Allowing non-string placeholders weakens the standard error model and makes error payloads inconsistent.

## How it works internally

In semantic call enforcement:
1. detect `err.internal` calls,
2. enforce single-argument shape,
3. enforce string message type,
4. emit deterministic diagnostics for malformed usage.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/ailang-core/src/semantic.rs`
  - semantic fixture:
    - `invalid_err_internal_argument_type.ai`
  - diagnostic-tag coverage:
    - `compiler/ailang-core/tests/diagnostic_tags.rs`
  - integration fixture alignment:
    - `compiler/ailang-core/tests/c_backend.rs`
    - `compiler/ailang-cli/tests/json_output.rs`
- Outputs:
  - compile-time rejection of non-string internal-error messages,
  - stable diagnostics for tooling and CI.
- Constraint:
  - runtime error ABI behavior remains unchanged.

## Failure modes and diagnostics

Example:
- `err.internal(1)` ->
  - `E4001`: err.internal message argument must be `String`.

## Example usage

```ailang
fn makeInternal() -> Int {
  err.internal("internal");
  0
}
```

## Tradeoffs and next steps

- Tradeoff: placeholder numeric internal-error messages accepted in early fixtures are now rejected.
- Next:
  - continue hardening the remaining constructor helpers (`err.validation`, `err.auth`, `err.notFound`, `err.conflict`, `err.rateLimit`) toward typed argument parity.
