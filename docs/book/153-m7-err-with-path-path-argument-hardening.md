# 153 M7 Slice: err.withPath Path-Argument Hardening

This chapter documents a focused M7 hardening step for `err.withPath` path typing.

## What it is

Added semantic checks for `err.withPath`:
- call shape must be `err.withPath(error, path)`,
- `path` must be `String`.

Violations emit `E4001` with `security` tags.

## Why it exists

The standard error model expects deterministic path strings (for example JSON-path-like values). Allowing placeholder numerics weakens diagnostics and makes error payload shaping inconsistent.

## How it works internally

In semantic call enforcement:
1. detect `err.withPath` calls,
2. enforce exact two-argument shape,
3. require `String` type for the path argument,
4. emit deterministic diagnostics with canonical usage guidance.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/ailang-core/src/semantic.rs`
  - semantic fixture:
    - `invalid_err_with_path_argument_type.ai`
  - diagnostic-tag coverage:
    - `compiler/ailang-core/tests/diagnostic_tags.rs`
  - integration fixture alignment:
    - `compiler/ailang-core/tests/c_backend.rs`
    - `compiler/ailang-cli/tests/json_output.rs`
- Outputs:
  - compile-time rejection of non-string `err.withPath` path values,
  - stable diagnostics for tooling and CI fixtures.
- Constraint:
  - runtime error ABI behavior remains unchanged.

## Failure modes and diagnostics

Example:
- `err.withPath(err, 2)` ->
  - `E4001`: err.withPath path argument must be `String`.

## Example usage

```ailang
fn attach(base: Int) -> Int {
  err.withPath(base, "$.field");
  0
}
```

## Tradeoffs and next steps

- Tradeoff: permissive placeholder path arguments in earlier bridge fixtures are now rejected.
- Next:
  - continue strict typing across remaining error-helper enrichers (`withLimit`, `withDependency`) for complete standard-error contract parity.
