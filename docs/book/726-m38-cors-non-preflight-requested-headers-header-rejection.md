# M38-S103 CORS Non-Preflight Requested-Headers Header Rejection

## What it is

M38-S103 hardens runtime CORS handling by rejecting non-preflight requests that include `Access-Control-Request-Headers`.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

`Access-Control-Request-Headers` is a preflight-only CORS negotiation header. Allowing it on non-preflight requests weakens protocol-shape enforcement and allows malformed requests to enter normal dispatch paths.

## How it works

1. In CORS-enabled non-preflight branch (`method != OPTIONS`), runtime checks for `Access-Control-Request-Headers`.
2. If present:
   - response is `400 Bad Request`,
   - body is `cors request requested headers header not allowed`,
   - CORS allow-origin header is omitted.
3. Existing non-preflight behavior remains unchanged for valid request shapes.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_non_preflight_with_requested_headers_header`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_origin_header_on_success_when_enabled`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_non_preflight_with_requested_method_header`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Runtime adds one strict protocol-shape branch for CORS-enabled non-preflight traffic.
- Non-conforming clients now fail fast with deterministic diagnostics instead of being silently tolerated.

## Next

1. Continue tightening CORS protocol-shape enforcement with deterministic malformed-input branches.
2. Keep incremental hardening slices aligned across runtime, e2e tests, roadmap, and book docs.
