# 154 M7 Slice: err.withLimit Argument Hardening

This chapter documents a focused M7 hardening step for typed limit-detail construction.

## What it is

Added semantic checks for `err.withLimit`:
- call shape must be `err.withLimit(error, name, max, actual)`,
- `name` must be `String`,
- `max` and `actual` must be numeric.

Violations emit `E4001` with `security` tags.

## Why it exists

`err.withLimit` is part of resource-limit error reporting. Permissive placeholders weaken reliability and make error payloads inconsistent with the standard error contract.

## How it works internally

In semantic call enforcement:
1. detect `err.withLimit` calls,
2. enforce exact four-argument shape,
3. validate name/max/actual argument types,
4. emit deterministic diagnostics for malformed limit payloads.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/ailang-core/src/semantic.rs`
  - semantic fixture:
    - `invalid_err_with_limit_argument_type.ai`
  - diagnostic-tag coverage:
    - `compiler/ailang-core/tests/diagnostic_tags.rs`
  - integration fixture alignment:
    - `compiler/ailang-core/tests/c_backend.rs`
    - `compiler/ailang-cli/tests/json_output.rs`
- Outputs:
  - compile-time rejection of invalid `err.withLimit` argument types,
  - stable diagnostics for CLI/editor tooling.
- Constraint:
  - runtime error ABI behavior remains unchanged.

## Failure modes and diagnostics

Example:
- `err.withLimit(err, 2, 3, 4)` ->
  - `E4001`: err.withLimit name argument must be `String`.

## Example usage

```ailang
fn attach(base: Int) -> Int {
  err.withLimit(base, "limit", 100, 120);
  0
}
```

## Tradeoffs and next steps

- Tradeoff: permissive placeholder arguments in early bridge fixtures now fail semantic checks.
- Next:
  - extend strict typing to `err.withDependency` and remaining enrichers for full standard-error helper parity.
