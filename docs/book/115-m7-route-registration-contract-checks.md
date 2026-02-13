# 115 M7 Slice: Route Registration Contract Checks

This chapter documents a semantic hardening slice for HTTP route wiring in M7.

## What it is

Added initial call-contract checks for `http.get` and `http.post`:
- route path argument must be `String`,
- handler argument must resolve to a declared function symbol,
- handler function must declare `effects { net }`.

## Why it exists

After enabling function-symbol handler references, route registration still accepted placeholder arguments. This slice moves route wiring toward real API intent and catches unsafe/invalid handler wiring earlier.

## How it works internally

During call analysis, intrinsic route calls are validated in semantic checks:
- arity check expects `(router, path, handler)`,
- path type check validates `String`,
- handler symbol resolution uses callable alias resolution and function catalog lookup,
- handler effect check validates declared `net` effect set.

The checks produce:
- `E4001` for route shape/path/symbol contract violations,
- `E4002` when handler is missing `net` effect.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/sec4-core/src/semantic.rs`
  - HTTP route-related fixtures in core/CLI tests
- Outputs:
  - stronger semantic diagnostics for invalid route registration
  - updated compile fixtures using explicit string paths and function handlers
- Constraint:
  - this is a v0.1-lite contract, not full handler signature typechecking yet.

## Failure modes and diagnostics

Examples:
- `http.post(router, 1, health)` -> `E4001` (`route path must be String`)
- `http.get(router, "/health", health)` where `health` lacks `effects { net }` -> `E4002`

## Example usage

```ut
fn health() effects { net } -> Int {
  res.text(200, "ok");
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  http.get(router, "/health", health);
  http.serve(8080, router);
  0
}
```

## Tradeoffs and next steps

- Tradeoff: route checks are focused and intentionally narrow in this slice.
- Next:
  - enforce richer handler signature compatibility against `Handler` shape,
  - then add real runtime request dispatch assertions once non-stub HTTP behavior lands.
