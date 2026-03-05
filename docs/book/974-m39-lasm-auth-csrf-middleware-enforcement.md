# M39: LASM Auth/CSRF Middleware Enforcement

## Why

`sec4 run --backend lasm` already enforced handler-level auth helpers, but router middleware contracts (`auth.withAuth`, `csrf.withCsrf`) were not enforced at request time.

That gap allowed middleware-protected routes to execute without the expected security checks in LASM mode.

## What Changed

1. Added LASM route-composition middleware extraction:
   - Detects `auth.withAuth(...)` usage in the entry call graph.
   - Detects `csrf.withCsrf(...)` usage in the entry call graph.
2. Route planning now tags routes with internal middleware markers:
   - auth-required marker for middleware-protected routes.
   - csrf-required marker for middleware-protected routes.
3. Added request-time LASM middleware enforcement:
   - Auth middleware rejection: deterministic `401 AUTH.UNAUTHORIZED`.
   - CSRF middleware rejection: deterministic `403 AUTH.CSRF_TOKEN_INVALID`.
4. Wired CSRF policy defaults into LASM runtime defaults:
   - `enabled` / mode-off behavior,
   - csrf cookie/header names,
   - protected-method normalization fallback (`POST,PUT,PATCH,DELETE`).
5. Internal middleware marker headers are now stripped before response emission.

## Validation

1. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_enforces_auth_with_auth_middleware`
2. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_enforces_csrf_with_csrf_middleware`
3. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_enforces_auth_require_role_with_ctx_current`
