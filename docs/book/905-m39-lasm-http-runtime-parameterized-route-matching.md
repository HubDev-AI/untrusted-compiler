# M39 - LASM HTTP Runtime Parameterized Route Matching

## What Was Added

Implemented parameterized route matching in LASM HTTP runtime.

Files:

1. `compiler/sec4-core/src/lasm_http_runtime.rs`

## Behavior

1. Runtime now supports path patterns like `/users/:id`.
2. Route resolution order is deterministic:
   - exact route match first,
   - pattern route match second.
3. Pattern matching currently enforces:
   - same method,
   - same segment count,
   - literal segments equal,
   - `:param` segments accept any single path segment.
4. Existing deterministic 404 and handler-result mapping behavior remains unchanged.

## Why

This moves LASM runtime beyond exact-path-only routing and aligns baseline behavior with realistic API route shapes, which is required for useful async backend evolution.

## Validation

1. `cargo test -p sec4-core lasm_http_runtime::tests::`
2. `cargo test -p sec4 --test commands lasm_smoke_command_`
3. `cargo test -p sec4 --test commands`

New runtime coverage:

- `parameterized_route_matches_path_segments_deterministically`
