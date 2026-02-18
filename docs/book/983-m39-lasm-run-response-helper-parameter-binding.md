# M39: LASM Run Response-Helper Parameter Binding

## Why

Response extraction for `run --backend lasm` followed helper call graphs but ignored caller argument bindings when entering helper functions.

That broke handlers that route responses through parameterized helper wrappers such as `emit(status, body)`.

## What Changed

1. Added caller-to-callee parameter binding propagation for response extraction helper traversal.
2. Updated recursive response extraction to enter helper functions with seeded argument bindings.
3. Switched response extraction function traversal to path-local recursion guards (insert/remove) so separate call sites can analyze the same helper with different argument bindings deterministically.
4. Added LASM oneshot integration coverage proving status/body literals passed through helper parameters are materialized correctly.

## Validation

1. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_resolves_param_bound_response_helper_calls`
2. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_uses_latest_response_write_in_handler`
