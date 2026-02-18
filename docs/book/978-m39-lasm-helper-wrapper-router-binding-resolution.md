# M39: LASM Helper Wrapper Router-Binding Resolution

## Why

Route middleware extraction already handled direct router wrappers and helper side-effects, but one call shape still dropped middleware markers:

- helper wrappers where the router argument is not the first call argument.

In those cases, middleware derivation could miss auth/csrf requirements when wrappers returned router parameters through helper indirection.

## What Changed

1. Updated helper-call middleware extraction to resolve helper call arguments through caller bindings before entering helper-body analysis.
2. Threaded caller binding context through helper middleware analysis sites so wrapper return expressions can resolve alias chains deterministically.
3. Added LASM run integration coverage for helper wrapper route registration where the router argument is not first (`wrapRoute(mode, router)`), proving auth middleware is still enforced.

## Validation

1. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_applies_helper_middleware_when_router_arg_is_not_first`
2. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_applies_helper_middleware_through_route_register_function`
3. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_does_not_apply_unbound_middleware_to_route`
