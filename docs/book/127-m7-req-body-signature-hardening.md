# 127 M7 Slice: `req.body` Signature Hardening

This chapter documents a focused M7 trust-boundary hardening step for raw body extraction.

## What it is

Added semantic call-shape enforcement for `req.body(...)`:
- call form must be `req.body(ctx, request)`,
- first argument must be `Ctx`,
- second argument must be `Request`.

Violations emit `E4001` with `security` + `schema` tags.

## Why it exists

`req.body` is the raw inbound trust boundary (`Untrusted<Bytes>` source). Leaving it permissive (`req.body(1, 2)` placeholders) weakens contract clarity and blurs how request context should flow through handlers.

This slice makes the boundary explicit without changing runtime lowering.

## How it works internally

In semantic intrinsic enforcement:
1. extended request-source contract checks with a dedicated `req.body` branch,
2. enforced exact arity (`2`) and typed argument checks (`Ctx`, `Request`),
3. emitted fix-oriented diagnostics with source/safety tags.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/sec4-core/src/semantic.rs`
  - semantic fixtures for missing-arg and wrong-type `req.body` calls
  - diagnostic-tag coverage in `compiler/sec4-core/tests/diagnostic_tags.rs`
  - req/res CLI `c-bin` integration fixture in `compiler/sec4-cli/tests/json_output.rs`
- Outputs:
  - compile-time rejection of malformed `req.body(...)` calls,
  - integration fixture now uses explicit `ctx`/`req` parameters.
- Constraint:
  - this slice hardens semantic contract only; existing runtime ABI call mapping is unchanged.

## Failure modes and diagnostics

Examples:
- `req.body(1)` -> `E4001` (`req.body expects (ctx, request) arguments`).
- `req.body(1, 2)` ->
  - `E4001`: first argument must be `Ctx`,
  - `E4001`: second argument must be `Request`.

## Example usage

```ut
fn decode(ctx: Ctx, req: Request, schema: Schema<Int>) effects { net } -> Int {
  let raw = req.body(ctx, req);
  raw;
  req.json(schema);
  0
}
```

## Tradeoffs and next steps

- Tradeoff: fixture and sample code must thread explicit `ctx`/`req` symbols where `req.body` is used.
- Next:
  - keep tightening remaining bridge contracts toward a fully typed handler surface,
  - use these tagged diagnostics in upcoming LSP quick-fix flows.
