# 396 M16 Slice: Live HTTP Runtime Serve Bootstrap

This chapter documents the M16-S1 slice that replaces HTTP runtime placeholders with an executable serve path for compiled Untrusted<T> binaries.

## What it is

A minimal HTTP runtime implementation in `runtime/c/sec4_runtime.c` + typed ABI updates in `runtime/c/sec4_runtime.h` for:

- router allocation and route registration (`GET`/`POST`),
- request dispatch into compiled handler functions,
- real response emission via `res.text(status, body)`,
- deterministic oneshot mode for CI/test harnesses.

## Why it exists

Compile-path parity was already present (`http.router/get/post/serve` lowered correctly), but runtime behavior was still no-op. That blocked true end-to-end serving with `sec4 run`.

## Implementation details

1. ABI typed signatures were added for runtime HTTP surface:
   - `sec4_rt_res_text(int64_t status, const char *body)`
   - `sec4_rt_http_route_get/post(int64_t router, const char *path, int64_t (*handler)(void))`
   - `sec4_rt_http_serve(int64_t port, int64_t router)`
   - middleware pass-through: `sec4_rt_with_* (int64_t router, int64_t cfg)`
2. Runtime now keeps an in-process router table (handle-based) with exact method/path matching.
3. `sec4_rt_http_serve` opens a TCP socket, accepts HTTP/1.1 requests, parses request line, routes, and writes deterministic responses.
4. Handler response state is captured through `sec4_rt_res_text`, then serialized as:
   - status line,
   - `Content-Type`,
   - `Content-Length`,
   - `Connection: close`.

## Runtime modes

- Default mode: serve loop (blocks and keeps serving requests).
- Test mode: `SEC4_RT_HTTP_SERVE_MODE=oneshot`
  - accepts at most one request,
  - exits on first request or timeout,
  - timeout configurable via `SEC4_RT_HTTP_SERVE_TIMEOUT_MS`.

## Failure behavior

- Invalid request line -> `400 Bad Request`.
- Missing route -> `404 Not Found`.
- Invalid port / bind/listen failure -> non-zero runtime return.

## Validation

- Updated `sec4-core` runtime ABI assertions (`compiler/sec4-core/tests/c_backend.rs`).
- Added full E2E CLI integration test (`compiler/sec4-cli/tests/json_output.rs`):
  - builds a temp project via `--emit c-bin`,
  - runs binary in oneshot mode,
  - sends `GET /health`,
  - asserts `HTTP/1.1 200 OK` and body `ok`.
- Existing run tests now pass deterministic oneshot env so they do not hang.

## Tradeoffs

- Scope is intentionally minimal: exact path matching and `res.text` response path only.
- JSON/HTML/full request-body runtime semantics remain for follow-up slices.
- Middleware behavior is pass-through in this slice (shape is preserved, enforcement later).
