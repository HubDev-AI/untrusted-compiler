# 118 M7 Slice: Route Handler Compatibility (Zero-Arg Bridge Contract)

This chapter documents the next M7 compatibility rule for route handlers in the current runtime bridge.

## What it is

Added a route-handler compatibility check for `http.get`/`http.post`:
- handler function must declare zero parameters in the current v0 runtime bridge.

## Why it exists

Route calls now accept function symbols, but parameterized handlers are not yet represented safely in the current C runtime bridge path. This rule prevents mismatched handler signatures from silently compiling through weak ABI assumptions.

## How it works internally

During route registration validation:
1. resolve handler symbol to a declared function,
2. inspect its function signature in the semantic catalog,
3. emit `E4001` if parameter count is non-zero.

The diagnostic includes:
- handler symbol name,
- declared parameter count,
- remediation note to perform request/schema reads inside the handler body.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/sec4-core/src/semantic.rs`
  - semantic fixtures and `examples/hello-api`
- Outputs:
  - compile-time rejection of parameterized route handlers in current bridge mode
- Constraints:
  - this is explicitly a bridge-stage contract; richer handler signatures can be introduced in later milestones with runtime/lowering support.

## Failure modes and diagnostics

Example failure:
- `http.post(router, "/users", createUser)` where `createUser(schema: Schema<Int>)`:
  - `E4001`: route handler must not declare parameters in v0 runtime bridge.

## Example usage

```ut
fn createUser() effects { net } -> Int {
  req.json("CreateUserRequest");
  res.ok(201, "CreateUserResponse", 1);
  0
}
```

## Tradeoffs and next steps

- Tradeoff: current handler model is intentionally restrictive to keep runtime bridge behavior explicit and deterministic.
- Next:
  - implement real handler signature model (`Ctx`, `Request` -> `Result<Response, HttpError>`) once runtime dispatch supports it,
  - then relax this temporary zero-arg restriction.
