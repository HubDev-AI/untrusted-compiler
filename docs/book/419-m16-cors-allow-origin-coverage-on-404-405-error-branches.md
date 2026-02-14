# 419 M16 Slice: CORS Allow-Origin Coverage on 404/405 Error Branches

This chapter documents M16-S21: expanding CORS allow-origin coverage to non-middleware runtime error branches.

## What it is

Two new runtime e2e tests that assert CORS allow-origin propagation for:

- `404 Not Found` (missing route)
- `405 Method Not Allowed` (path matched, method mismatched)

when `cors.withCors(...)` is enabled.

## Why it exists

CORS coverage already existed for preflight, success responses, and middleware rejection paths. Generic runtime error branches (`404`/`405`) remained uncovered.

This slice prevents regressions where only some error branches carry CORS headers.

## Implementation details

1. Added `c_bin_http_runtime_applies_cors_origin_header_on_not_found_when_enabled`:
   - CORS-enabled router,
   - request to missing path,
   - asserts `404` + `Access-Control-Allow-Origin: *`.
2. Added `c_bin_http_runtime_applies_cors_origin_header_on_405_when_enabled`:
   - CORS-enabled router with POST-only route,
   - GET request to same path,
   - asserts `405` + `Allow: POST` + `Access-Control-Allow-Origin: *`.

## Validation

Executed locally:

- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_origin_header_on_not_found_when_enabled`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_origin_header_on_405_when_enabled`
- `cargo test -p sec4 --test json_output`
- `cargo test -p sec4-core --test c_backend`

All passed.

## Tradeoffs and next steps

- CORS propagation coverage now spans success, preflight, middleware rejects, and generic routing error branches.
- Next CORS-focused work should move from propagation coverage to stricter policy/runtime coupling (for example credentials + explicit-origin enforcement at runtime path configuration).
