# M38-S107 CORS Preflight Authorization-Header Rejection

## What it is

M38-S107 hardens runtime CORS preflight handling by rejecting preflight requests that include an `Authorization` header.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

CORS preflight is a negotiation request and should stay credential-free. Accepting authorization-bearing preflight requests permits malformed request shapes that browsers do not emit and weakens deterministic protocol enforcement.

## How it works

1. In CORS preflight branch (`method == OPTIONS`), runtime checks whether `Authorization` header is present.
2. If present:
   - response is `400 Bad Request`,
   - body is `cors preflight authorization header not allowed`,
   - preflight allow-methods header block is omitted.
3. Cookie-header and body guards remain active and order-preserving in the same preflight path.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_with_authorization_header`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_with_cookie_header`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_handles_cors_preflight_when_enabled`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Runtime adds one strict header-presence guard in the preflight branch.
- Clients sending non-browser preflight shapes with authorization now fail fast with deterministic diagnostics.

## Next

1. Continue tightening malformed CORS request-shape rejection in incremental runtime hardening slices.
2. Keep e2e/runtime/docs alignment deterministic per slice.
