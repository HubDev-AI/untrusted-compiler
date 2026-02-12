# 119 M7 Slice: Route Handler Return-Type Bridge Contract

This chapter documents a small M7 compatibility tightening for handler functions used in route registration.

## What it is

Added semantic enforcement that route handlers used by `http.get`/`http.post` must return numeric types (`Int` or `Int64`) in the current runtime bridge stage.

## Why it exists

The runtime bridge currently treats handler symbols as numeric-return functions in generated C flow. Allowing arbitrary return types at semantic level can hide ABI mismatches. This rule keeps compile-time behavior aligned with current runtime assumptions.

## How it works internally

After resolving handler symbols during route registration checks, semantic analysis now inspects handler signature return type and emits `E4001` when return type is not numeric.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/ailang-core/src/semantic.rs`
  - semantic fixture `invalid_http_route_handler_return_not_numeric.ai`
- Outputs:
  - compile-time rejection of route handlers returning non-numeric types in bridge mode.
- Constraint:
  - this is a bridge-stage restriction and can be relaxed once full typed handler ABI is implemented.

## Failure modes and diagnostics

Example:
- Handler `fn health() -> Bool` used in `http.get(...)`:
  - `E4001`: route handler must return `Int`/`Int64` in v0 runtime bridge.

## Example usage

```ailang
fn health() effects { net } -> Int {
  res.text(200, "ok");
  0
}
```

## Tradeoffs and next steps

- Tradeoff: this is stricter than long-term language intent, but preserves deterministic bridge compatibility.
- Next:
  - migrate to typed handler ABI (`Ctx`, `Request` -> `Result<Response, HttpError>`) in later runtime milestones and retire temporary bridge restrictions.
