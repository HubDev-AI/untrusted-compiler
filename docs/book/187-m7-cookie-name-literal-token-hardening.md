# 187 M7 Slice: Cookie Name Literal Token Hardening

This chapter documents a focused M7 hardening step for constant `cookie.build(...)` name literals.

## What it is

`cookie.build(name, value)` now rejects name literals containing invalid token characters. Allowed literal characters are ASCII alphanumerics plus `-` and `_`.

Violations emit `E4001` diagnostics tagged with `security` + `sink`.

## Why it exists

Cookie constructor checks already enforced arity/type and value CR/LF guards. This slice adds static name-token validation for constant cookie names to reduce malformed `Set-Cookie` construction.

## How it works internally

In `enforce_cookie_build_signature(...)`:
1. existing argument shape/type checks run first,
2. for string-literal name arguments, token characters are validated,
3. invalid name literals emit a dedicated sink diagnostic.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/sec4-core/src/semantic.rs`
  - semantic fixture: `invalid_cookie_build_name_literal_characters.ut`
  - tag test coverage in `compiler/sec4-core/tests/diagnostic_tags.rs`
- Outputs:
  - compile-time rejection for malformed cookie-name literals.
- Constraint:
  - this is literal-only validation; dynamic names still rely on runtime/cookie builder validation.

## Failure modes and diagnostics

Example:
- `cookie.build("sess ion;", "ok")` -> `E4001`: cookie.build name literal contains invalid characters.

## Example usage

```ut
fn ok() -> Int {
  cookie.build("session_id", "abc123");
  0
}
```

## Tradeoffs and next steps

- Tradeoff: strict literal token checks reject non-standard cookie-name forms during compile-time.
- Next:
  - extend cookie literal diagnostics with offending-character snippets for clearer fix guidance.
