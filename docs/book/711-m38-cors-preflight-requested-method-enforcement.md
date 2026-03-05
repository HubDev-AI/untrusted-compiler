# M38-S88 CORS Preflight Requested-Method Enforcement

## What it is

M38-S88 adds requested-method enforcement in runtime CORS preflight handling. Preflight requests now validate `Access-Control-Request-Method` against the configured CORS allow-methods list before returning success preflight headers.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Before this slice, preflight handling returned `204` with allow-methods headers even when the requested method was not permitted. That allowed invalid preflight requests to receive success responses instead of deterministic policy rejection.

## How it works

1. In the `OPTIONS` preflight path, runtime reads `Access-Control-Request-Method`.
2. Runtime resolves effective allow-methods list from router policy state.
3. If requested method is present and not in allow-methods CSV:
   - returns `403 Forbidden`,
   - emits deterministic body: `cors preflight method not allowed`,
   - does not emit the normal preflight allow-method/header block.
4. If requested method is allowed (or missing), behavior remains unchanged:
   - `204 No Content` with CORS preflight headers.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_when_requested_method_is_not_allowed`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_handles_cors_preflight_when_enabled`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_allows_cors_preflight_with_auth_and_csrf_enabled`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Adds a stricter rejection branch for malformed/disallowed preflight requests.
- Slightly increases preflight logic complexity, but closes a real runtime policy-enforcement gap.

## Next

1. Continue hardening CORS preflight request validation for remaining request-controlled fields.
2. Keep policy/runtime parity slices strict: reject invalid request inputs deterministically and preserve success-path behavior for valid requests.
