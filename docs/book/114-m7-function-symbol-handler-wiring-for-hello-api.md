# 114 M7 Slice: Function Symbol Handler Wiring for `hello-api`

This chapter documents an M7 slice that moves `examples/hello-api` from placeholder route arguments to explicit handler wiring with real route paths.

## What it is

Implemented two linked changes:
- Semantic analyzer now accepts declared function symbols as value expressions (for example passing `health` as a route handler argument).
- `examples/hello-api/src/main.ut` now registers:
  - `GET /health` -> `health`
  - `POST /users` -> `createUser`
  and composes middleware in a policy-shaped chain.

## Why it exists

M7 requires a concrete API-shaped vertical slice, not only intrinsic call stubs with numeric placeholders. This change makes the sample service readable as a real router definition while still compiling through the current C-runtime bridge.

## How it works internally

In semantic analysis (`analyze_expr`), identifier resolution now follows this order:
1. Local scope bindings (`let`, params).
2. Declared function symbols (treated as valid value expressions).
3. Otherwise emit `N3003` unknown identifier.

This keeps strict unknown-name diagnostics while enabling handler-style wiring APIs that pass function names as values.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/sec4-core/src/semantic.rs`
  - `examples/hello-api/src/main.ut`
  - integration + semantic fixtures
- Outputs:
  - `hello-api` generated C now contains explicit route paths and handler symbols.
- Constraints:
  - Handler signature compatibility is not fully enforced yet (v0.1-lite behavior).
  - Runtime HTTP serving remains stub-backed in this stage.

## Failure modes and diagnostics

- Unknown non-function identifiers still raise `N3003`.
- Regressions in handler-symbol acceptance are covered by:
  - semantic golden fixture `valid_function_symbol_handler_reference.ut`
  - CLI c-bin integration for `hello-api`.

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

- Tradeoff: this enables practical route wiring now, but does not yet typecheck full handler signatures.
- Next steps:
  - add `Handler`-shape signature checks for route registration,
  - then move M7 runtime from bridge stubs to real request dispatch and JSON decode behavior tests.
