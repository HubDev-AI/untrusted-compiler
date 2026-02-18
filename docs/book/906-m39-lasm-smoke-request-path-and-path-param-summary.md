# M39 - LASM Smoke Request-Path and Path-Param Summary

## What Was Added

Extended LASM runtime exchange metadata and `sec4 lasm-smoke` input/output surface.

Files:

1. `compiler/sec4-core/src/lasm_http_runtime.rs`
2. `compiler/sec4-cli/src/main.rs`
3. `compiler/sec4-cli/tests/commands.rs`

## Behavior

1. LASM HTTP runtime now carries captured path params in each completed exchange.
   - `HttpExchange` includes `path_params`.
   - Parameterized routes (`/users/:id`) capture named segments (`id=42`).
2. `sec4 lasm-smoke` now accepts optional `--request-path`.
   - Route registration still uses `--route` (pattern/contract).
   - Request submission uses `--request-path` when provided (default remains `--route`).
3. `lasm-smoke` summaries now include captured params:
   - text mode: `pathParams=...`
   - json mode: `pathParams` object + `requestPath` field.

## Why

This makes parameterized routing observable and testable through CLI smoke runs. Without `--request-path`, parameterized route probing cannot submit concrete values against route patterns.

## Validation

1. `cargo test -p sec4-core lasm_http_runtime::tests::`
2. `cargo test -p sec4 --test commands lasm_smoke_command_`
3. `cargo test -p sec4 --test commands`

New coverage:

- `lasm_smoke_command_captures_path_params_with_request_path_override`
