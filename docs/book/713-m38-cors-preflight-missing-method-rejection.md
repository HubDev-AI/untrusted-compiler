# M38-S90 CORS Preflight Missing-Method Rejection

## What it is

M38-S90 hardens runtime CORS preflight handling by rejecting preflight requests that omit the required `Access-Control-Request-Method` header.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

CORS preflight requires the client to declare the requested method. Without that header, runtime cannot apply allow-method policy deterministically. Accepting such requests as valid preflight weakens policy enforcement and can hide client misconfiguration.

## How it works

1. In runtime `OPTIONS` preflight path, read `Access-Control-Request-Method`.
2. If missing:
   - return `400 Bad Request`,
   - emit deterministic body: `cors preflight missing requested method`,
   - skip normal preflight allow-method/header block.
3. If present, keep existing behavior:
   - enforce allow-method list,
   - allow valid preflight or reject disallowed method.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_without_requested_method_header`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_when_requested_method_is_not_allowed`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_allows_cors_preflight_with_auth_and_csrf_enabled`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Stricter validation may reject previously tolerated malformed preflight requests.
- Behavior becomes more explicit and deterministic for clients and tests.

## Next

1. Continue hardening request-policy boundary checks for CORS/security middleware with deterministic diagnostics.
2. Keep every runtime behavior hardening slice paired with direct e2e coverage and book/roadmap traceability.
