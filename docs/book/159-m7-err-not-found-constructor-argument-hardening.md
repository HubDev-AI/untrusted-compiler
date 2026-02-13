# 159 M7 Slice: err.notFound Constructor-Argument Hardening

This chapter documents a focused M7 hardening step for typed not-found error constructor usage.

## What it is

Added semantic checks for `err.notFound`:
- call shape must be `err.notFound(code, message)`,
- `code` must be `String`,
- `message` must be `String`.

Violations emit `E4001` with `security` tags.

## Why it exists

Not-found errors are part of the public error contract. Permissive placeholder constructor values weaken consistency and make response semantics less reliable.

## How it works internally

In semantic call enforcement:
1. detect `err.notFound` calls,
2. enforce exact two-argument shape,
3. enforce string code/message types,
4. emit deterministic diagnostics with usage guidance.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/sec4-core/src/semantic.rs`
  - semantic fixture:
    - `invalid_err_not_found_argument_type.ut`
  - diagnostic-tag coverage:
    - `compiler/sec4-core/tests/diagnostic_tags.rs`
  - integration fixture alignment:
    - `compiler/sec4-core/tests/c_backend.rs`
    - `compiler/sec4-cli/tests/json_output.rs`
- Outputs:
  - compile-time rejection of malformed not-found constructor values,
  - stable diagnostics for tooling and CI.
- Constraint:
  - runtime error ABI behavior remains unchanged.

## Failure modes and diagnostics

Example:
- `err.notFound(1, 2)` ->
  - `E4001`: code/message arguments must be `String`.

## Example usage

```ut
fn makeNotFound() -> Int {
  err.notFound("RESOURCE.NOT_FOUND", "missing");
  0
}
```

## Tradeoffs and next steps

- Tradeoff: permissive constructor placeholders from early bridge fixtures now fail semantic checks.
- Next:
  - continue constructor typing hardening for `err.conflict` and `err.rateLimit`.
