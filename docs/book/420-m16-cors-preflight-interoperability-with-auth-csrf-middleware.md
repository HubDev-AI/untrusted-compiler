# 420 M16 Slice: CORS Preflight Interoperability with Auth + CSRF Middleware

This chapter documents M16-S22: ensuring CORS preflight works when CORS, auth, and CSRF middleware are composed.

## What it is

A new runtime e2e test that verifies `OPTIONS` preflight requests remain accepted when all three middleware layers are active:

- `cors.withCors`
- `csrf.withCsrf`
- `auth.withAuth`

## Why it exists

Preflight handling was already covered for CORS alone, but not for composed security middleware chains.

Without this test, middleware ordering or gate logic changes could accidentally route preflight requests into auth/csrf reject paths.

## Implementation details

1. Added `c_bin_http_runtime_allows_cors_preflight_with_auth_and_csrf_enabled`.
2. Fixture composes middleware in runtime route setup (`cors -> csrf -> auth`).
3. Test sends `OPTIONS /users` with standard preflight headers and asserts:
   - `HTTP/1.1 204 No Content`
   - `Access-Control-Allow-Origin: *`
   - `Access-Control-Allow-Methods` and `Access-Control-Allow-Headers` presence.

## Validation

Executed locally:

- `cargo test -p sec4 --test json_output c_bin_http_runtime_allows_cors_preflight_with_auth_and_csrf_enabled`
- `cargo test -p sec4 --test json_output`
- `cargo test -p sec4-core --test c_backend`

All passed.

## Tradeoffs and next steps

- Middleware composition coverage now includes preflight interoperability in addition to success and error branches.
- Next composition-hardening slice can extend to policy-coupled runtime behavior (for example stricter credential/origin runtime combinations).
