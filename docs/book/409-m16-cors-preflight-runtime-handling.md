# 409 M16 Slice: CORS Preflight Runtime Handling

This chapter documents M16-S14: adding runtime CORS preflight handling for routers that enable CORS middleware.

## What it is

A runtime update in `runtime/c/sec4_runtime.c` that:

- marks routers as CORS-enabled when `sec4_rt_with_cors(...)` is called,
- intercepts `OPTIONS` requests for CORS-enabled routers,
- emits deterministic preflight response:
  - `HTTP/1.1 204 No Content`
  - `Access-Control-Allow-Origin: *`
  - `Access-Control-Allow-Methods: GET, POST, PUT, PATCH, DELETE, OPTIONS`
  - `Access-Control-Allow-Headers: content-type, authorization`
  - `Access-Control-Max-Age: 600`

## Why it exists

Before this slice, CORS middleware calls were pass-through only. Runtime had no preflight branch, so CORS-enabled services could not demonstrate baseline browser-facing behavior in live integration tests.

## Implementation details

1. Extended router runtime state with `cors_enabled`.
2. `sec4_rt_with_cors(router, cfg)` now marks the router as CORS-enabled.
3. In request handling:
   - when method is `OPTIONS` and router is CORS-enabled,
   - runtime returns deterministic preflight response without route-handler execution.

## Validation

Added integration test in `compiler/sec4-cli/tests/json_output.rs`:

- `c_bin_http_runtime_handles_cors_preflight_when_enabled`

Flow:
1. Build fixture with `cors.withCors(...)`.
2. Send `OPTIONS /users` with preflight headers.
3. Assert deterministic `204` + required CORS allow headers.

## Tradeoffs and next steps

- This slice is a deterministic baseline; policy-driven origin/method/header filtering is not implemented yet.
- Future slices can connect policy file values and enforce stricter CORS safety constraints (credentials/wildcard coupling, `Vary: Origin`, allowlists).
