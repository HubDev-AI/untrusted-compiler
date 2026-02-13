# 155 M7 Slice: err.withDependency Argument Hardening

This chapter documents a focused M7 hardening step for typed dependency-detail construction.

## What it is

Added semantic checks for `err.withDependency`:
- call shape must be `err.withDependency(error, name, operation, retryable)`,
- `name` must be `String`,
- `operation` must be `String`,
- `retryable` must be `Bool`.

Violations emit `E4001` with `security` tags.

## Why it exists

Dependency error metadata should be deterministic and typed. Permissive placeholder arguments hide malformed dependency details and weaken the standard error contract.

## How it works internally

In semantic call enforcement:
1. detect `err.withDependency` calls,
2. enforce exact four-argument shape,
3. validate dependency/operation/retryable argument types,
4. emit deterministic diagnostics for invalid payload shapes.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/ailang-core/src/semantic.rs`
  - semantic fixture:
    - `invalid_err_with_dependency_argument_type.ai`
  - diagnostic-tag coverage:
    - `compiler/ailang-core/tests/diagnostic_tags.rs`
  - integration fixture alignment:
    - `compiler/ailang-core/tests/c_backend.rs`
    - `compiler/ailang-cli/tests/json_output.rs`
- Outputs:
  - compile-time rejection of malformed dependency-detail arguments,
  - stable diagnostics for tooling and CI.
- Constraint:
  - runtime error ABI behavior remains unchanged.

## Failure modes and diagnostics

Example:
- `err.withDependency(err, 2, 3, 4)` ->
  - `E4001`: err.withDependency name argument must be `String`.

## Example usage

```ailang
fn attach(base: Int) -> Int {
  err.withDependency(base, "postgres", "query", true);
  0
}
```

## Tradeoffs and next steps

- Tradeoff: placeholder dependency fields accepted in early bridge fixtures are now rejected.
- Next:
  - continue strict typing across remaining error-helper constructors (`validation`, `auth`, etc.) if/when bridge signatures are promoted from placeholder-friendly forms.
