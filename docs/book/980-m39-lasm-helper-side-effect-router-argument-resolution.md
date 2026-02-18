# M39: LASM Helper Side-Effect Router-Argument Resolution

## Why

LASM middleware side-effect propagation previously assumed the router argument was always first in helper calls.

That missed valid helper shapes such as:

- `applyAuth(mode, router)`

where middleware is applied inside the helper to the router parameter.

## What Changed

1. Added call-target router argument resolution helper for middleware propagation paths.
2. Direct middleware calls (`auth.withAuth`, `csrf.withCsrf`) still resolve router from argument index `0`.
3. Helper calls now resolve router argument index by parameter type (`Router`) in the callee function signature, with deterministic fallback to argument `0`.
4. Applied this resolution in both:
   - nested middleware extraction while traversing helper bodies,
   - live route-registration back-propagation when middleware side-effects are discovered.

## Validation

1. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_applies_helper_side_effect_middleware_when_router_arg_is_not_first`
2. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_applies_auth_middleware_wrapped_in_serve_call`
3. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_does_not_apply_unbound_middleware_to_route`
