# 421 M16 Slice: Security Headers Coverage on Auth/CSRF Rejection Paths

This chapter documents M16-S23: extending security-header coverage to middleware rejection responses.

## What it is

Two runtime e2e tests that verify `sec.withSecurityHeaders(...)` remains active when requests are rejected by middleware:

- auth reject path (`401 Unauthorized`)
- csrf reject path (`403 Forbidden`)

## Why it exists

Security-header coverage previously focused on success and not-found branches. Middleware rejection branches can carry sensitive error metadata and should keep the same hardened header baseline.

This slice prevents regressions that would strip security headers from auth/csrf rejection responses.

## Implementation details

1. Added `c_bin_http_runtime_applies_security_headers_on_auth_reject_when_enabled`.
2. Added `c_bin_http_runtime_applies_security_headers_on_csrf_reject_when_enabled`.
3. Both tests assert deterministic presence of:
   - `X-Content-Type-Options: nosniff`
   - `X-Frame-Options: DENY`
   - `Referrer-Policy: strict-origin-when-cross-origin`
4. Tests also assert expected deterministic error codes (`AUTH.UNAUTHORIZED`, `AUTH.CSRF_TOKEN_INVALID`).

## Validation

Executed locally:

- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_on_auth_reject_when_enabled`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_on_csrf_reject_when_enabled`
- `cargo test -p sec4 --test json_output`
- `cargo test -p sec4-core --test c_backend`

All passed.

## Tradeoffs and next steps

- Security-header behavior is now covered across success, not-found, and middleware rejection paths.
- Next header-focused hardening can add explicit runtime coverage for `405` and preflight branches under security-header middleware, if required by policy posture.
