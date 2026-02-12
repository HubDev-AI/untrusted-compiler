# 125 M7 Slice: Request Source Signature Hardening

This chapter documents a focused M7 trust-boundary hardening step for request source helpers.

## What it is

Added semantic call-shape enforcement for:
- `req.query(name)`
- `req.pathParam(name)`
- `req.header(name)`

Each call now requires:
- exactly one argument,
- argument type `String`.

Violations emit `E4001` with `security` + `schema` tags.

## Why it exists

These APIs are trust-boundary sources (`Untrusted<String>` outputs), but they still accepted permissive placeholder call shapes. Tightening signatures reduces ambiguity and aligns call sites with explicit key-based extraction semantics.

## How it works internally

In semantic intrinsic enforcement:
1. added `enforce_request_source_signatures(...)`,
2. recognized canonical call names for query/path/header variants,
3. enforced exact arity and `String` key type checks,
4. produced fix-oriented diagnostics for invalid call shapes.

`req.body(...)` is intentionally left unchanged in this slice to avoid premature bridge assumptions around request/context parameterization.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/ailang-core/src/semantic.rs`
  - semantic fixtures for invalid query/header call forms
  - diagnostic tag coverage in `compiler/ailang-core/tests/diagnostic_tags.rs`
  - req/res and cors-origin CLI `c-bin` integration fixtures
- Outputs:
  - compile-time rejection of malformed request-source key calls,
  - integration fixtures now use string keys (`"q"`, `"id"`, `"authorization"`, `"origin"`).
- Constraint:
  - this slice targets signature shape only; deeper request object typing is staged separately.

## Failure modes and diagnostics

Examples:
- `req.query("q", "extra")` -> `E4001` (`req.query expects exactly one argument`).
- `req.header(1)` -> `E4001` (`req.header argument must be String`).

## Example usage

```ailang
fn decode(schema: Schema<Int>) effects { net } -> Int {
  let q = req.query("q");
  let id = req.pathParam("id");
  let auth = req.header("authorization");
  q;
  id;
  auth;
  req.json(schema);
  0
}
```

## Tradeoffs and next steps

- Tradeoff: existing placeholder-heavy integration fixtures must be normalized to string-key call forms.
- Next:
  - progressively tighten remaining request/response helper contracts,
  - keep trust-boundary diagnostics consistently tagged for future LSP quick-fixes.
