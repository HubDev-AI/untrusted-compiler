# 186 M7 Slice: Cookie Value CRLF Literal Hardening

This chapter documents a focused M7 hardening step for constant `cookie.build(...)` value literals.

## What it is

`cookie.build(name, value)` now rejects value literals containing CR/LF sequences (`\r`, `\n`, or escaped forms) at compile time.

Violations emit `E4001` diagnostics tagged with `security` + `sink`.

## Why it exists

Cookie helper signature checks already enforce typed string arguments. This slice adds static literal validation to prevent response-splitting sequences in constant cookie values.

## How it works internally

In `enforce_cookie_build_signature(...)`:
1. existing call-shape and string-type checks run first,
2. for string-literal value arguments, the analyzer scans for CR/LF markers,
3. flagged literals emit a dedicated sink diagnostic.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/sec4-core/src/semantic.rs`
  - semantic fixture: `invalid_cookie_build_value_crlf_literal.ut`
  - tag test coverage in `compiler/sec4-core/tests/diagnostic_tags.rs`
- Outputs:
  - compile-time rejection for CR/LF cookie value literals.
- Constraint:
  - this check is literal-only; dynamic values still rely on runtime/cookie builder validation.

## Failure modes and diagnostics

Example:
- `cookie.build("session", "\\n")` -> `E4001`: cookie.build value literal cannot contain CR/LF.

## Example usage

```ut
fn ok() -> Int {
  cookie.build("session", "abc123");
  0
}
```

## Tradeoffs and next steps

- Tradeoff: conservative escaped-sequence detection may reject some intentionally escaped literal forms.
- Next:
  - extend cookie constructor literal checks to validate name token constraints in constant literals.
