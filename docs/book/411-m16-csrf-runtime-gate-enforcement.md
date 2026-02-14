# 411 M16 Slice: CSRF Runtime Gate Enforcement

This chapter documents M16-S16: enabling runtime CSRF checks when `csrf.withCsrf(...)` middleware is active.

## What it is

A runtime update in `runtime/c/sec4_runtime.c` that enforces double-submit CSRF checks for protected methods:

- `POST`
- `PUT`
- `PATCH`
- `DELETE`

Expected request inputs:

- header: `X-CSRF-Token`
- cookie: `csrf=<token>`

Tokens must match.

## Why it exists

CSRF middleware was previously pass-through. This slice turns it into an active runtime security boundary and aligns with the project’s strict backend-hardening direction.

## Implementation details

1. Extended router state with `csrf_enabled`.
2. `sec4_rt_with_csrf(router, cfg)` now enables CSRF enforcement for router.
3. Added header/cookie parsing helpers for request validation.
4. For protected methods:
   - missing or mismatched token results in deterministic structured error:
     - status `403`
     - code `AUTH.CSRF_TOKEN_INVALID`
     - message `CSRF token missing or invalid`

## Validation

Added runtime integration tests in `compiler/sec4-cli/tests/json_output.rs`:

- `c_bin_http_runtime_rejects_post_without_csrf_tokens_when_enabled`
- `c_bin_http_runtime_allows_post_with_matching_csrf_tokens_when_enabled`

These cover both reject and allow branches.

## Tradeoffs and next steps

- This is a deterministic baseline double-submit implementation.
- Future slices can connect policy-configurable CSRF modes and richer cookie/token naming while preserving current safe defaults.
