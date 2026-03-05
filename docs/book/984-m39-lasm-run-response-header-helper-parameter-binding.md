# M39: LASM Run Response-Header Helper Parameter Binding

## Why

LASM response-header extraction traversed helper call graphs, but did not propagate caller argument values into helper parameter bindings.

This missed deterministic header extraction for helper wrappers like:

- `emitHeader(name, value)` calling `res.setHeader(headers.name(name), headers.value(value))`.

## What Changed

1. Added caller-to-callee string binding propagation for response-header extraction helper traversal.
2. Updated helper traversal recursion guard to path-local insert/remove for deterministic multi-callsite analysis.
3. Added LASM oneshot integration coverage proving helper-parameter header literals are emitted in final responses.

## Validation

1. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_resolves_param_bound_response_header_helper_calls`
2. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_resolves_param_bound_response_helper_calls`
