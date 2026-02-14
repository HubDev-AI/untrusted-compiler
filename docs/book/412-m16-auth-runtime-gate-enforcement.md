# 412 M16 Slice: Auth Runtime Gate Enforcement

This chapter documents M16-S17: enabling runtime auth checks when `auth.withAuth(...)` middleware is active.

## What it is

A runtime update in `runtime/c/sec4_runtime.c` that enforces bearer-auth header checks on auth-enabled routers:

- required header: `Authorization`
- required shape: `Bearer <token>`

Requests that fail this check receive deterministic structured `401` errors.

## Why it exists

Auth middleware was previously pass-through at runtime. This slice makes runtime behavior match security expectations by rejecting unauthenticated requests early in the dispatch path.

## Implementation details

1. Extended router state with `auth_enabled`.
2. `sec4_rt_with_auth(router, cfg)` now enables auth checks for the router.
3. Runtime request path now enforces for non-`OPTIONS` methods:
   - header present,
   - bearer format with non-empty token.
4. Failure path emits deterministic standard error envelope:
   - status `401`
   - code `AUTH.UNAUTHORIZED`
   - message `Authorization header missing or invalid`

## Validation

Added integration tests in `compiler/sec4-cli/tests/json_output.rs`:

- `c_bin_http_runtime_rejects_request_without_auth_header_when_enabled`
- `c_bin_http_runtime_allows_request_with_auth_header_when_enabled`

These validate both reject and allow branches through live runtime HTTP execution.

## Tradeoffs and next steps

- This is intentionally a baseline header-shape gate; token verification is out of scope for runtime bootstrap.
- Future slices can integrate policy-driven auth modes and principal propagation into request context.
