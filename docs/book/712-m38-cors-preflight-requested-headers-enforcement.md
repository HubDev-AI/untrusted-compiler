# M38-S89 CORS Preflight Requested-Headers Enforcement

## What it is

M38-S89 hardens runtime CORS preflight validation by enforcing `Access-Control-Request-Headers` against configured allow-headers policy.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Preflight handling already validated requested method (M38-S88), but did not validate requested headers from `Access-Control-Request-Headers`. That allowed unsupported headers to receive a success preflight response instead of deterministic policy rejection.

## How it works

1. Added runtime helper to validate requested headers CSV:
   - trims tokens,
   - requires valid header-name tokens,
   - requires each token to exist in allow-headers policy list.
2. Updated preflight path:
   - when request includes `Access-Control-Request-Headers`, runtime validates it against configured/default allow-headers list,
   - rejects invalid/disallowed requested headers with `403 Forbidden`,
   - emits deterministic body: `cors preflight headers not allowed`.
3. Preserved success path:
   - allowed requested headers continue to return `204` with standard CORS preflight headers.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_allows_cors_preflight_when_requested_headers_are_allowed`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_when_requested_header_is_not_allowed`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_handles_cors_preflight_when_enabled`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_allows_cors_preflight_with_auth_and_csrf_enabled`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Adds one more branch in preflight processing for request-driven header validation.
- Stricter rejection behavior may surface previously hidden client misconfigurations, which is intentional.

## Next

1. Continue closing remaining request-policy runtime parity gaps in preflight/sink boundary behavior.
2. Keep each hardening slice coupled to deterministic runtime e2e tests and roadmap/book traceability.
