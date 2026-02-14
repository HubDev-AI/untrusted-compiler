# 413 M16 Slice: CORS Allow-Origin Propagation on Runtime Responses

This chapter documents M16-S18: propagating CORS allow-origin headers to normal runtime responses (not only preflight).

## What it is

A runtime update in `runtime/c/sec4_runtime.c` that applies:

- `Access-Control-Allow-Origin: *`

to non-preflight responses when `cors.withCors(...)` middleware is active.

## Why it exists

M16-S14 introduced preflight handling, but normal responses did not include allow-origin, which left CORS behavior incomplete for browser clients. This slice closes that gap with deterministic middleware-driven response header propagation.

## Implementation details

1. Added CORS header block into runtime response-header composition for normal dispatch branches.
2. Kept preflight behavior unchanged and deterministic.
3. Preserved existing interactions with:
   - trace header,
   - security headers middleware,
   - method-mismatch and error branches.

## Validation

Added integration test in `compiler/sec4-cli/tests/json_output.rs`:

- `c_bin_http_runtime_applies_cors_origin_header_on_success_when_enabled`

The test verifies:

- `HTTP/1.1 200 OK`
- presence of `Access-Control-Allow-Origin: *` on normal response path.

## Tradeoffs and next steps

- Current propagation uses wildcard allow-origin baseline.
- Future slices can introduce policy-driven origin allowlists, `Vary: Origin`, and credentials coupling rules for stricter posture.
