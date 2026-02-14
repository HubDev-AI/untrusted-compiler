# 422 M16 Slice: Security Headers Coverage on 405 and Preflight Branches

This chapter documents M16-S24: extending security-header coverage to dispatch and preflight branches.

## What it is

Two new runtime e2e tests that assert `sec.withSecurityHeaders(...)` is applied on:

- `405 Method Not Allowed` branch
- CORS preflight `204 No Content` branch

## Why it exists

Security-header coverage already existed for success, not-found, and middleware rejection paths. Dispatch/preflight branches were still untested.

This slice removes that gap so runtime header hardening is covered across all main HTTP response categories.

## Implementation details

1. Added `c_bin_http_runtime_applies_security_headers_on_405_when_enabled`:
   - security middleware + method mismatch request,
   - asserts `405`, `Allow`, and security headers.
2. Added `c_bin_http_runtime_applies_security_headers_on_cors_preflight_when_enabled`:
   - composed CORS + security middleware,
   - asserts `204` preflight + CORS headers + security headers.

## Validation

Executed locally:

- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_on_405_when_enabled`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_on_cors_preflight_when_enabled`
- `cargo test -p sec4 --test json_output`
- `cargo test -p sec4-core --test c_backend`

All passed.

## Tradeoffs and next steps

- Security-header runtime coverage now spans success, not-found, middleware rejects, dispatch mismatch, and preflight branches.
- Next middleware-hardening slice can focus on stricter policy/runtime coupling behavior rather than branch-level coverage gaps.
