# 397 M16 Slice: JSON Response Materialization for Runtime Routes

This chapter documents M16-S2: enabling minimal runtime JSON response behavior for route handlers that use `res.ok`/`res.json`.

## What it is

A runtime bridge extension in `runtime/c/sec4_runtime.c` that materializes JSON HTTP responses for:

- `sec4_rt_res_ok(...)`
- `sec4_rt_res_json(...)`
- `sec4_rt_res_ok_meta(...)`

This builds on M16-S1 router/serve loop work.

## Why it exists

After M16-S1, only `res.text` produced concrete HTTP responses. API-style handlers such as `/users` already compiled and executed, but their JSON intrinsics still behaved as no-op stubs. M16-S2 closes that gap with deterministic JSON response output.

## Implementation details

1. Added shared runtime helper `sec4_rt_store_response(...)` to centralize:
   - status,
   - content-type,
   - bounded response-body copying.
2. `res.text` now uses the shared helper.
3. Implemented minimal JSON response materialization:
   - `res.ok` -> `201 Created`, `application/json; charset=utf-8`, body `{"ok":true}`
   - `res.json` -> `200 OK`, `application/json; charset=utf-8`, body `{"ok":true}`
   - `res.okMeta` -> `201 Created`, `application/json; charset=utf-8`, body `{"ok":true,"meta":{}}`
4. `req.json` remains a bridge-stage runtime placeholder (successful decode contract), keeping semantic policy/type guarantees as the primary gate in v0.

## Inputs, outputs, constraints

- Input: route handlers lowered to runtime intrinsics (for example `req.json("CreateUserRequest")`, `res.ok(...)`).
- Output: deterministic HTTP JSON response via the M16-S1 socket server.
- Constraint: payloads are currently fixed bridge outputs, not full schema-driven serialization.

## Failure modes and diagnostics

- Route not found still returns `404 Not Found`.
- Invalid request line still returns `400 Bad Request`.
- Socket bind/listen failures still return non-zero runtime status.
- JSON payload shape is intentionally fixed in this slice; value-level serialization is deferred.

## Validation

Added CLI integration test in `compiler/sec4-cli/tests/json_output.rs`:

- `c_bin_http_runtime_serves_users_post_with_json_response_when_clang_available`

Flow:
1. Build a temporary fixture project through `--emit c-bin`.
2. Start compiled binary in oneshot mode.
3. Send `POST /users` with JSON body.
4. Assert response contains:
   - `HTTP/1.1 201 Created`
   - `Content-Type: application/json; charset=utf-8`
   - body `{"ok":true}`

## Tradeoffs and next steps

- This slice prioritizes deterministic end-to-end serving over full JSON semantics.
- Next runtime slices should add:
  - schema-aware encode/decode behavior,
  - request-body mapping integration with `req.json` decode constraints,
  - richer success envelope shaping (`StdSuccess<T>`) with typed value serialization.
