# 117 M7 Slice: HTTP Call-Shape Contract Checks

This chapter captures the next M7 semantic hardening step for HTTP intrinsic call shapes.

## What it is

Added explicit semantic validation for HTTP intrinsic argument types:
- `http.get` / `http.post` now require `Router` as argument 1.
- `http.serve` now requires:
  - numeric port (`Int`/`Int64`) as argument 1,
  - `Router` as argument 2.

## Why it exists

Route registration handler/path contracts were already enforced, but router/serve argument shapes were still permissive. This slice closes that gap and makes HTTP wiring more predictable and auditable.

## How it works internally

During intrinsic call checks:
- route registration validation now verifies `Router` type for argument 1.
- a dedicated `http.serve` validator checks arity and typed `(port, router)` contract.
- violations emit deterministic `E4001` diagnostics with concrete fix notes.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/ailang-core/src/semantic.rs`
  - semantic fixtures for invalid route/serve call shapes
- Outputs:
  - compile-time rejection of malformed route/serve calls
- Constraint:
  - these are semantic shape checks only; runtime serving behavior remains bridge/stub based in current M7 stage.

## Failure modes and diagnostics

Examples:
- `http.get(1, "/health", health)` -> route first arg not `Router` (`E4001`).
- `http.serve("8080", router)` -> non-numeric port (`E4001`).

## Example usage

```ailang
fn main() effects { net } -> Int {
  let router = http.router();
  http.get(router, "/health", health);
  http.serve(8080, router);
  0
}
```

## Tradeoffs and next steps

- Tradeoff: checks are intentionally strict for HTTP call shape while runtime behavior is still evolving.
- Next:
  - add stronger req/res call contracts and budget-aware decode rules,
  - then implement runtime behavior tests for middleware/preflight/error paths.
