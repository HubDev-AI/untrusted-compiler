# 123 M7 Slice: Header/Cookie Sink Signature Hardening

This chapter documents a focused M7 sink-contract hardening step for HTTP header and cookie response APIs.

## What it is

Added semantic call-shape enforcement for:
- `res.setHeader(name, value)`:
  - exactly two arguments,
  - `name` must be `HeaderName`,
  - `value` must be `HeaderValue`.
- `res.addCookie(cookie)`:
  - exactly one argument,
  - argument must be `Cookie`.

All violations emit `E4001` and are tagged `security` + `sink`.

## Why it exists

Before this slice, compile-path fixtures could still pass placeholder numerics into header/cookie sinks. That bypasses the typed primitive posture required by the security-first baseline and weakens compile-time guarantees against header misuse.

## How it works internally

In semantic intrinsic enforcement:
1. added `enforce_header_cookie_signatures(...)`,
2. routed call checks through canonical call-name helpers:
   - `is_set_header_call(...)`,
   - `is_add_cookie_call(...)`,
3. applied arity + type checks with fix-oriented notes,
4. preserved existing taint/secret sink checks and layered typed-contract diagnostics on top.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/ailang-core/src/semantic.rs`
  - semantic fixtures for invalid header/cookie argument types
  - diagnostic-tag coverage in `compiler/ailang-core/tests/diagnostic_tags.rs`
  - header/cookie `c-bin` integration fixture in `compiler/ailang-cli/tests/json_output.rs`
- Outputs:
  - compile-time rejection of malformed `res.setHeader` and `res.addCookie` calls,
  - integration fixture updated to valid typed flow:
    - `headers.name(...)`,
    - `headers.value(...)`,
    - `cookie.build(...)`.
- Constraint:
  - this enforces signature/type shape only; deeper runtime header policy (for example CR/LF rejection) remains owned by validation/runtime gates.

## Failure modes and diagnostics

Examples:
- `res.setHeader(1, 2)` ->
  - `E4001`: name must be `HeaderName`,
  - `E4001`: value must be `HeaderValue`.
- `res.addCookie(1)` ->
  - `E4001`: argument must be `Cookie`.

Secret/taint sink diagnostics remain active and can co-emit with signature diagnostics when both rules are violated.

## Example usage

```ailang
fn configure() effects { net } -> Int {
  let name = headers.name("X-Test");
  let value = headers.value("ok");
  let cookie = cookie.build("session", "token");
  res.setHeader(name, value);
  res.addCookie(cookie);
  0
}
```

## Tradeoffs and next steps

- Tradeoff: stricter sink signatures may produce extra diagnostics in older fixtures that relied on permissive placeholders.
- Next:
  - continue tightening remaining req/res contracts toward fully typed HTTP surface consistency,
  - keep adding sink-focused diagnostic tags to support future LSP quick-fix UX.
