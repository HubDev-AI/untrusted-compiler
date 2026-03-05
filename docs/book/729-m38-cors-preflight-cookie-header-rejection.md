# M38-S106 CORS Preflight Cookie-Header Rejection

## What it is

M38-S106 hardens runtime CORS preflight handling by rejecting preflight requests that include a `Cookie` header.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

CORS preflight requests are negotiation probes and should remain credential-free. Accepting cookie-bearing preflight requests broadens malformed request shapes and weakens deterministic enforcement.

## How it works

1. In CORS preflight branch (`method == OPTIONS`), runtime checks whether `Cookie` header is present.
2. If present:
   - response is `400 Bad Request`,
   - body is `cors preflight cookie header not allowed`,
   - preflight allow-methods header block is omitted.
3. Existing preflight missing/invalid/allow checks continue unchanged for cookie-free preflight traffic.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_with_cookie_header`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_with_request_body`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_handles_cors_preflight_when_enabled`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Runtime adds one strict cookie-presence branch in the preflight path.
- Non-browser or malformed clients that attach cookies to preflight now fail fast with deterministic diagnostics.

## Next

1. Continue tightening deterministic malformed-input rejection in CORS request-shape handling.
2. Keep runtime behavior, e2e tests, roadmap, and chapter updates synchronized by slice.
