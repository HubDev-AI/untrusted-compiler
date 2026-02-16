# M38-S108 CORS Non-Preflight Private-Network Header Rejection

## What it is

M38-S108 hardens runtime CORS handling by rejecting non-preflight requests that include `Access-Control-Request-Private-Network`.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

`Access-Control-Request-Private-Network` is a preflight negotiation header used in Private Network Access flow. Accepting it on non-preflight traffic allows malformed protocol shapes into normal request handling.

## How it works

1. In CORS-enabled non-preflight branch (`method != OPTIONS`), runtime checks for `Access-Control-Request-Private-Network` header presence.
2. If present:
   - response is `400 Bad Request`,
   - body is `cors request private-network header not allowed`,
   - CORS allow-origin header is omitted.
3. Existing non-preflight guards for origin/requested-method/requested-headers remain unchanged.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_non_preflight_with_requested_private_network_header`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_origin_header_on_success_when_enabled`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_non_preflight_with_requested_headers_header`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Runtime adds one additional strict header-presence branch in non-preflight CORS handling.
- Non-browser malformed requests fail fast with deterministic diagnostics.

## Next

1. Continue malformed CORS request-shape hardening slices with deterministic runtime diagnostics.
2. Keep runtime/test/docs updates synchronized per slice.
