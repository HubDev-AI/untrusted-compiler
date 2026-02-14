# 399 M16 Slice: Runtime Request JSON Gate for Invalid Payload Handling

This chapter documents M16-S4: connecting `req.json(...)` to live request body data in the C runtime so malformed payloads fail deterministically at runtime.

## What it is

A runtime update in `runtime/c/sec4_runtime.c` that introduces request-context tracking and a bridge-stage JSON gate in:

- `sec4_rt_req_json(schema)`

The runtime now inspects the inbound HTTP body and records JSON gate status for the current request.

## Why it exists

Before this slice, `req.json(...)` always succeeded at runtime as a placeholder. That allowed malformed bodies to continue through handler success paths, which blocked realistic end-to-end behavior for schema-gated input routes.

## Implementation details

1. Added runtime request state (`g_sec4_rt_request`) with:
   - method/path snapshot,
   - request body buffer,
   - JSON gate flags (`json_checked`, `json_valid`).
2. Extended HTTP client handling to:
   - parse `Content-Length`,
   - read additional bytes when headers indicate more body data,
   - store bounded request body bytes for gate evaluation.
3. Implemented bridge-level JSON shape check:
   - accept trimmed payloads that look like JSON object/array,
   - reject empty or malformed payloads.
4. `sec4_rt_req_json(...)` now returns deterministic 400 response payloads on gate failure:
   - missing body: `{"error":"JSON body required"}`
   - invalid shape: `{"error":"invalid json body"}`
5. Success responders (`sec4_rt_res_json`, `sec4_rt_res_ok`, `sec4_rt_res_ok_meta`) now preserve gate failures and avoid overwriting the 400 response for failed `req.json(...)`.
6. Kept runtime header ABI compatibility for mixed lowered call sites by retaining no-prototype bridge declarations in `sec4_runtime.h`.

## Validation

Added CLI integration test in `compiler/sec4-cli/tests/json_output.rs`:

- `c_bin_http_runtime_req_json_rejects_invalid_body_when_clang_available`

Flow:
1. Build temp project via `--emit c-bin`.
2. Route calls `req.json("CreateUserRequest")` then `res.ok(...)`.
3. Send invalid JSON body (`not-json`) to `POST /users`.
4. Assert response contains:
   - `HTTP/1.1 400 Bad Request`
   - deterministic payload `{"error":"invalid json body"}`.

## Tradeoffs and next steps

- This is intentionally a lightweight bridge-level shape check, not full JSON parsing/validation.
- Full schema-aware decode with typed extraction remains a later runtime slice.
