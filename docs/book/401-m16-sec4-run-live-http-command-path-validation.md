# 401 M16 Slice: sec4 run Live HTTP Command-Path Validation

This chapter documents M16-S6: validating the full `sec4 run` command path with a real HTTP roundtrip.

## What it is

A CLI integration-test expansion in `compiler/sec4-cli/tests/json_output.rs` that proves `sec4 run` is not only compiling a binary, but also executing the runtime HTTP server and serving requests in oneshot mode.

## Why it exists

Earlier M16 slices validated runtime behavior by launching compiled binaries directly. That proved runtime correctness, but it did not explicitly verify the same behavior through the public command path developers use day-to-day: `sec4 run`.

## Implementation details

1. Added test:
   - `run_command_serves_http_route_in_oneshot_mode_when_clang_available`.
2. Test flow:
   - create a temporary project with route `POST /users` using `req.json(...)` and `res.ok(...)`,
   - launch `sec4 run --path <temp-project>` with:
     - `SEC4_RT_HTTP_SERVE_MODE=oneshot`
     - `SEC4_RT_HTTP_SERVE_TIMEOUT_MS=10000`
   - open a TCP connection to the configured port and send the HTTP request,
   - assert deterministic response contract.
3. Deterministic safeguards:
   - polling loop fails fast if command exits before request,
   - bounded wait loops prevent hanging tests.

## Validation

Assertions in the new test verify:

- `HTTP/1.1 201 Created`
- `Content-Type: application/json; charset=utf-8`
- body `{"ok":true}`
- command exits successfully in oneshot mode after handling the request.

## Tradeoffs and next steps

- This slice strengthens confidence in the public CLI execution path without introducing new runtime complexity.
- Future slices can extend command-path coverage to additional routes (`GET /health`, error paths, middleware behavior) and replay/capture integration.
