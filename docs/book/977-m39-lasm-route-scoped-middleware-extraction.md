# M39: LASM Route-Scoped Middleware Extraction

## Why

LASM middleware extraction was previously global across the entry call graph.

That meant an unrelated `auth.withAuth(...)` or `csrf.withCsrf(...)` call could incorrectly force auth/csrf enforcement on routes registered through a different router.

## What Changed

1. Extended route registration metadata with route-scoped middleware flags:
   - `require_auth_middleware`
   - `require_csrf_middleware`
2. Route registration matching now inspects the router argument expression (`http.<method>(router, path, handler)`), resolves local aliases, and derives middleware requirements from that specific router chain.
3. Added router-wrapper helper awareness for route-scoped extraction:
   - when route router expressions call local helper functions, middleware extraction now follows helper return expressions using parameter-to-argument bindings.
   - when helper calls are used as side effects (`attachAuth(router);`), middleware state now propagates to the affected router binding for later route registration.
   - helper-call parameter binding now resolves through caller aliases before analysis, so wrappers with non-first router parameters (for example `wrapRoute(mode, router)`) preserve middleware requirements deterministically.
4. Removed legacy global middleware extraction pass.
5. LASM route planning now applies auth/csrf internal markers from per-route middleware flags only.

## Validation

1. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_does_not_apply_unbound_middleware_to_route`
2. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_applies_auth_middleware_from_helper_call_side_effect`
3. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_enforces_auth_with_auth_middleware`
4. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_enforces_csrf_with_csrf_middleware`
5. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_skips_auth_middleware_when_auth_mode_env_off`
6. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_skips_csrf_middleware_when_csrf_mode_env_off`
7. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_applies_helper_middleware_when_router_arg_is_not_first`
