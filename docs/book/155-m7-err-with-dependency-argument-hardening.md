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
  - `compiler/sec4-core/src/semantic.rs`
  - semantic fixture:
    - `invalid_err_with_dependency_argument_type.ut`
  - diagnostic-tag coverage:
    - `compiler/sec4-core/tests/diagnostic_tags.rs`
  - integration fixture alignment:
    - `compiler/sec4-core/tests/c_backend.rs`
    - `compiler/sec4-cli/tests/json_output.rs`
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

```ut
fn attach(base: StdError) -> StdError {
  err.withDependency(base, "postgres", "query", true)
}
```

## Tradeoffs and next steps

- Tradeoff: placeholder dependency fields accepted in early bridge fixtures are now rejected.
- Next:
  - strict `err.withDependency` error-argument typing is covered in a follow-up slice (`169-m7-err-with-dependency-error-argument-hardening.md`).
