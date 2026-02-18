# M39 - LASM Smoke Helper Call-Graph Route Resolution

## What Was Added

Extended `sec4 lasm-smoke` route planning to resolve realistic handler wiring patterns, not only direct calls in `main`.

Files:

1. `compiler/sec4-cli/src/main.rs`
2. `compiler/sec4-cli/tests/commands.rs`

## Behavior

`sec4 lasm-smoke` now performs recursive AST-based discovery for two pieces of execution metadata:

1. Route handler binding for `http.<method>(...)` registrations.
2. Deterministic response extraction from response helpers:
   - `res.text(status, body)`
   - `res.html(html)`
   - `res.json(schema, value)` and `res.json(status, schema, value)`
   - `res.ok(status, schema, value)`
   - `res.okMeta(status, schema, value, meta)`

The resolver now:

1. walks function call graphs starting from the LASM entry function,
2. follows helper function calls while guarding cycles with a visited-function set,
3. supports route method mapping for `GET/POST/PUT/PATCH/DELETE/OPTIONS/HEAD`,
4. finds response helper calls in nested helper functions (not only direct handler body statements).

Summary output includes `status=<code>` from the first response so helper-derived status extraction is visible in deterministic smoke output.

If no route/response plan is discovered, `lasm-smoke` keeps deterministic fallback behavior (`origin=entry`).

## Why

Real projects frequently move route registration and response helpers into shared functions. The previous direct-only scan underfit that structure and reduced smoke signal quality for multi-file/module-style programs.

## Validation

1. `cargo test -p sec4 --test commands lasm_smoke_command_`
2. `cargo test -p sec4 --test commands`

New coverage:

- `lasm_smoke_command_resolves_routes_and_response_through_helper_calls`
- `lasm_smoke_command_extracts_res_ok_status_and_body`
