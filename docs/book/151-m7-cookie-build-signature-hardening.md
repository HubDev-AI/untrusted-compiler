# 151 M7 Slice: cookie.build Signature Hardening

This chapter documents a focused M7 hardening step for typed cookie-constructor usage.

## What it is

Added semantic checks for `cookie.build`:
- call shape must be exactly `cookie.build(name, value)`,
- `name` must be `String`,
- `value` must be `String`.

Violations emit `E4001` with `security` tags.

## Why it exists

`cookie.build` is the canonical constructor for cookie sink values. Allowing arbitrary placeholder types weakens typed header/cookie safety and makes invalid cookie assembly pass too far through the pipeline.

## How it works internally

In semantic call enforcement:
1. detect `cookie.build` intrinsic calls,
2. enforce exact two-argument shape,
3. validate both arguments are `String`,
4. emit deterministic `E4001` diagnostics for malformed cookie-constructor usage.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/sec4-core/src/semantic.rs`
  - semantic fixture:
    - `invalid_cookie_build_argument_type.ut`
  - diagnostic-tag coverage:
    - `compiler/sec4-core/tests/diagnostic_tags.rs`
  - integration fixture alignment:
    - `compiler/sec4-core/tests/c_backend.rs`
    - `compiler/sec4-cli/tests/json_output.rs`
- Outputs:
  - compile-time rejection of non-string cookie-constructor inputs,
  - stable diagnostics for editor/CLI consumers.
- Constraint:
  - runtime ABI remains unchanged; this slice hardens semantic contracts only.

## Failure modes and diagnostics

Example:
- `cookie.build(1, "value")` ->
  - `E4001`: cookie.build name argument must be `String`.

## Example usage

```ut
fn configure() effects { net } -> Int {
  let cookie = cookie.build("session", "token")
  res.addCookie(cookie)
  0
}
```

## Tradeoffs and next steps

- Tradeoff: placeholder numeric cookie-builder arguments used in early bootstrap fixtures are now rejected.
- Next:
  - extend cookie-builder typing with policy-aligned attributes (for example SameSite/Secure flags) as the runtime model matures.
