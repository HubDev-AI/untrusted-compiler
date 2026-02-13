# 158 M7 Slice: err.auth Constructor-Argument Hardening

This chapter documents a focused M7 hardening step for typed auth-error constructor usage.

## What it is

Added semantic checks for `err.auth`:
- call shape must be `err.auth(code, message, status)`,
- `code` must be `String`,
- `message` must be `String`,
- `status` must be numeric.

Violations emit `E4001` with `security` tags.

## Why it exists

Auth errors are externally visible and policy-sensitive. Placeholder constructor values reduce reliability and can blur status/message semantics.

## How it works internally

In semantic call enforcement:
1. detect `err.auth` calls,
2. enforce exact three-argument shape,
3. enforce typed code/message/status arguments,
4. emit deterministic diagnostics with canonical usage guidance.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/sec4-core/src/semantic.rs`
  - semantic fixture:
    - `invalid_err_auth_argument_type.ut`
  - diagnostic-tag coverage:
    - `compiler/sec4-core/tests/diagnostic_tags.rs`
  - integration fixture alignment:
    - `compiler/sec4-core/tests/c_backend.rs`
    - `compiler/sec4-cli/tests/json_output.rs`
- Outputs:
  - compile-time rejection of malformed auth-error constructor values,
  - stable diagnostics for tooling and CI.
- Constraint:
  - runtime error ABI behavior remains unchanged.

## Failure modes and diagnostics

Example:
- `err.auth(1, 2, 401)` ->
  - `E4001`: code/message arguments must be `String`.

## Example usage

```ut
fn makeAuth() -> Int {
  err.auth("AUTH.FORBIDDEN", "forbidden", 401);
  0
}
```

## Tradeoffs and next steps

- Tradeoff: permissive constructor placeholders from early bridge fixtures now fail semantic checks.
- Next:
  - continue constructor typing hardening for `err.notFound`, `err.conflict`, and `err.rateLimit`.
