# 418 M16 Slice: CORS Allow-Origin Coverage on Middleware Rejection Paths

This chapter documents M16-S20: locking CORS allow-origin behavior on auth/csrf middleware reject branches.

## What it is

Two new runtime end-to-end tests that verify `Access-Control-Allow-Origin: *` is preserved when requests are rejected by middleware:

- auth reject path (`401 Unauthorized`)
- csrf reject path (`403 Forbidden`)

## Why it exists

CORS propagation was already covered for preflight and success responses, but middleware rejection branches were not explicitly locked.

Without these tests, regressions could silently remove allow-origin headers from security-relevant error responses.

## Implementation details

1. Added `c_bin_http_runtime_applies_cors_origin_header_on_auth_reject_when_enabled`:
   - combines `cors.withCors` + `auth.withAuth`,
   - sends unauthenticated request,
   - asserts `401` + allow-origin + deterministic auth error envelope.
2. Added `c_bin_http_runtime_applies_cors_origin_header_on_csrf_reject_when_enabled`:
   - combines `cors.withCors` + `csrf.withCsrf`,
   - sends missing-token POST request,
   - asserts `403` + allow-origin + deterministic csrf error envelope.
3. Stabilized run-command oneshot e2e loop used in full-suite execution by widening timeout/retry window and handling early-success exit without immediate panic.

## Validation

Executed locally:

- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_origin_header_on_auth_reject_when_enabled`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_origin_header_on_csrf_reject_when_enabled`
- `cargo test -p sec4 --test json_output`
- `cargo test -p sec4-core --test c_backend`

All passed.

## Tradeoffs and next steps

- Coverage now spans CORS preflight, success, and middleware rejection paths.
- Next runtime hardening can focus on policy-driven CORS constraints (credentials/wildcard coupling) rather than header propagation gaps.
