# M39: LASM Serve-Wrapper Middleware Propagation

## Why

LASM route middleware markers were derived from router expressions at route registration time.

That missed a valid composition shape where middleware is applied later at serve time, for example:

- register routes on `router`
- call `http.serve(port, auth.withAuth(router, authCfg))`

Without propagation, LASM could incorrectly serve those routes without auth middleware enforcement.

## What Changed

1. Route registration metadata now tracks the router binding identity used at registration time.
2. Middleware side-effect tracking now runs for all call expressions (including nested calls), not only statement-level calls.
3. When middleware is applied to a router binding after routes are already registered, LASM back-propagates auth/csrf requirements onto prior registrations that share that router binding.
4. This includes nested serve-call wrappers like `http.serve(..., auth.withAuth(router, ...))`.

## Validation

1. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_applies_auth_middleware_wrapped_in_serve_call`
2. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_applies_helper_middleware_when_router_arg_is_not_first`
3. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_does_not_apply_unbound_middleware_to_route`
