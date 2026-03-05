# M38-S100 CORS Preflight Body Rejection

## What it is

M38-S100 hardens runtime CORS handling by rejecting preflight requests that carry non-empty request bodies.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

CORS preflight is a metadata negotiation request and should not require request-body semantics. Accepting bodies on preflight increases parser surface and ambiguity for middleware behavior.

## How it works

1. Runtime preflight branch now checks parsed request body length.
2. If preflight body length is non-zero:
   - response is `400 Bad Request`,
   - body is `cors preflight body not allowed`.
3. Existing preflight logic remains unchanged for empty-body requests:
   - origin checks,
   - requested-method checks,
   - requested-headers checks,
   - success `204` header emission.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_with_request_body`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_handles_cors_preflight_when_enabled`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_when_requested_method_is_not_allowed`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Runtime preflight path adds one strict body guard.
- Some non-conforming clients that send preflight bodies now fail fast with deterministic diagnostics.

## Next

1. Keep tightening CORS protocol-shape enforcement with deterministic reject paths.
2. Continue shipping each hardening increment as runtime + e2e + roadmap/book bundle.
