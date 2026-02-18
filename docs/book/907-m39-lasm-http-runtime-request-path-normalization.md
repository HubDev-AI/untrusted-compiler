# M39 - LASM HTTP Runtime Request Path Normalization

## What Was Added

Extended LASM runtime route resolution to normalize incoming request paths before matching.

Files:

1. `compiler/sec4-core/src/lasm_http_runtime.rs`

## Behavior

1. Route matching now ignores query strings and fragments for both exact and parameterized route resolution.
   - Example: `/users/42?expand=1#tab` matches `/users/:id`.
2. Captured path params are extracted from normalized path segments.
3. Existing deterministic resolution order is preserved:
   - exact route first,
   - pattern route second.

## Why

Without normalization, realistic request paths containing query/fragment components miss otherwise valid routes, reducing runtime correctness for API-like traffic.

## Validation

1. `cargo test -p sec4-core lasm_http_runtime::tests::`
2. `cargo test -p sec4 --test commands lasm_smoke_command_`
3. `cargo test -p sec4 --test commands`

New runtime coverage:

- `route_matching_ignores_query_and_fragment_in_request_path`
