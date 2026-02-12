# 121 M7 Slice: req/res Call-Shape Signature Hardening

This chapter documents a focused M7 signature-contract hardening step for request/response helpers.

## What it is

Added semantic call-shape enforcement for two core APIs:
- `req.json` must be called as `req.json(schema)` (exactly one argument),
- `res.text` must be called as `res.text(status, body)` where:
  - `status` is numeric (`Int`/`Int64`),
  - `body` is `String`.

## Why it exists

Bridge-stage tests still allowed placeholder call forms (`res.text(200, 1)` and extra-arg `req.json(...)`), which weakens type intent and makes future runtime behavior harder to lock down. This slice makes these contracts explicit at compile time.

## How it works internally

In semantic intrinsic enforcement:
1. `req.json`:
   - missing arg check (existing),
   - new exact-arity check,
   - schema-argument shape check (from previous slice).
2. `res.text`:
   - exact-arity check,
   - numeric status check,
   - string body check.

All violations emit deterministic `E4001` diagnostics.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/ailang-core/src/semantic.rs`
  - semantic fixtures for `req.json` arity and `res.text` body type
  - req/res CLI integration fixture updates
- Outputs:
  - compile-time rejection of malformed `req.json`/`res.text` calls
  - updated integration expectations (`res.text(..., "ok")`)
- Constraint:
  - still bridge-stage behavior; richer type-safe response/body models can be layered later.

## Failure modes and diagnostics

Examples:
- `req.json("Schema", 1)` -> `E4001` (`schema gate expects exactly one schema argument`).
- `res.text(200, 1)` -> `E4001` (`res.text body must be String`).

## Example usage

```ailang
fn createUser() effects { net } -> Int {
  req.json("CreateUserRequest");
  res.text(200, "ok");
  0
}
```

## Tradeoffs and next steps

- Tradeoff: strict bridge call-shape checks may require updating older placeholder-heavy fixtures.
- Next:
  - continue tightening req/res contracts (including header/cookie call signatures),
  - then back these semantics with concrete runtime behavior tests for HTTP middleware and decode paths.
