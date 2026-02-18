# M39: LASM Route-Scoped Middleware Extraction

## Why

LASM middleware extraction was previously global across the entry call graph.

That meant an unrelated `auth.withAuth(...)` or `csrf.withCsrf(...)` call could incorrectly force auth/csrf enforcement on routes registered through a different router.

## What Changed

1. Extended route registration metadata with route-scoped middleware flags:
   - `require_auth_middleware`
   - `require_csrf_middleware`
2. Route registration matching now inspects the router argument expression (`http.<method>(router, path, handler)`), resolves local aliases, and derives middleware requirements from that specific router chain.
3. Removed legacy global middleware extraction pass.
4. LASM route planning now applies auth/csrf internal markers from per-route middleware flags only.

## Validation

1. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_does_not_apply_unbound_middleware_to_route`
2. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_enforces_auth_with_auth_middleware`
3. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_enforces_csrf_with_csrf_middleware`
4. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_skips_auth_middleware_when_auth_mode_env_off`
5. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_skips_csrf_middleware_when_csrf_mode_env_off`
