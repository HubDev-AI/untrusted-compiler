# 410 M16 Slice: Security Headers Runtime Injection

This chapter documents M16-S15: enabling runtime security-header injection when `sec.withSecurityHeaders(...)` is used.

## What it is

A runtime update in `runtime/c/sec4_runtime.c` that:

- tracks security-header middleware enablement per router,
- injects baseline security headers into live HTTP responses.

Injected headers:

- `X-Content-Type-Options: nosniff`
- `X-Frame-Options: DENY`
- `Referrer-Policy: strict-origin-when-cross-origin`

## Why it exists

Security-header middleware calls previously had no runtime effect. This slice makes the middleware meaningful in end-to-end runtime behavior and aligns with security-first defaults expected for backend services.

## Implementation details

1. Extended router state with `security_headers_enabled`.
2. `sec4_rt_with_security_headers(router, cfg)` now marks router as enabled.
3. Added runtime header-composition path so middleware headers are merged into outgoing responses, including:
   - successful route responses,
   - not-found error responses (`404`),
   - and other response branches sharing runtime serializer path.

## Validation

Added integration tests in `compiler/sec4-cli/tests/json_output.rs`:

- `c_bin_http_runtime_applies_security_headers_on_success_when_enabled`
- `c_bin_http_runtime_applies_security_headers_on_not_found_when_enabled`

Both validate presence of all three security headers on `200` and `404` responses.

## Tradeoffs and next steps

- This slice uses deterministic fixed header values as baseline.
- Future slices can map policy-driven header values and extend coverage to additional branches (`400`, `405`, preflight paths) with environment-aware behavior.
