# 160 M7 Slice: err.conflict Constructor-Argument Hardening

This chapter documents a focused M7 hardening step for typed conflict-error constructor usage.

## What it is

Added semantic checks for `err.conflict`:
- call shape must be `err.conflict(code, message)`,
- `code` must be `String`,
- `message` must be `String`.

Violations emit `E4001` with `security` tags.

## Why it exists

Conflict errors are part of the standardized runtime error model. Permissive placeholder values weaken compatibility and make downstream handling less deterministic.

## How it works internally

In semantic call enforcement:
1. detect `err.conflict` calls,
2. enforce exact two-argument shape,
3. enforce string code/message types,
4. emit deterministic diagnostics with canonical guidance.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/ailang-core/src/semantic.rs`
  - semantic fixture:
    - `invalid_err_conflict_argument_type.ai`
  - diagnostic-tag coverage:
    - `compiler/ailang-core/tests/diagnostic_tags.rs`
  - integration fixture alignment:
    - `compiler/ailang-core/tests/c_backend.rs`
    - `compiler/ailang-cli/tests/json_output.rs`
- Outputs:
  - compile-time rejection of malformed conflict constructor values,
  - stable diagnostics for tooling and CI.
- Constraint:
  - runtime error ABI behavior remains unchanged.

## Failure modes and diagnostics

Example:
- `err.conflict(1, 2)` ->
  - `E4001`: code/message arguments must be `String`.

## Example usage

```ailang
fn makeConflict() -> Int {
  err.conflict("RESOURCE.CONFLICT", "conflict");
  0
}
```

## Tradeoffs and next steps

- Tradeoff: permissive constructor placeholders from early bridge fixtures now fail semantic checks.
- Next:
  - complete constructor typing hardening for `err.rateLimit`.
