# M39 - LASM HTTP Runtime HEAD Fallback

## What Was Added

Extended LASM runtime route resolution with deterministic HEAD fallback behavior.

Files:

1. `compiler/sec4-core/src/lasm_http_runtime.rs`

## Behavior

1. Route lookup first resolves by exact method (`HEAD` stays `HEAD` when present).
2. If a `HEAD` request has no matching `HEAD` route, runtime falls back to `GET` route resolution for the same normalized path.
3. Fallback applies for both exact and parameterized routes because it reuses shared route-plan resolution.
4. Existing deterministic route order remains intact for each method:
   - exact route first,
   - pattern route second.

## Why

This improves practical HTTP compatibility for generated services while preserving deterministic routing semantics.

## Validation

1. `cargo test -p sec4-core lasm_http_runtime::tests::`
2. `cargo test -p sec4 --test commands lasm_smoke_command_`
3. `cargo test -p sec4 --test commands`

New runtime coverage:

- `head_request_falls_back_to_get_route_when_head_missing`
