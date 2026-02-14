# 400 M16 Slice: Runtime req.json Content-Type Gate Enforcement

This chapter documents M16-S5: enforcing request media-type constraints in the live runtime `req.json(...)` gate.

## What it is

A runtime hardening update in `runtime/c/sec4_runtime.c` where `sec4_rt_req_json(...)` now rejects non-JSON request media types.

Accepted media types:

- `application/json`
- `application/*+json`

Rejected media types now return deterministic `415 Unsupported Media Type`.

## Why it exists

M16-S4 introduced body-shape validation, but content-type remained unchecked. That allowed routes to pass `req.json(...)` with misleading or unsafe media-type declarations (for example `text/plain`), which is weaker than security-first request boundary behavior.

## Implementation details

1. Added content-type parsing during HTTP request capture:
   - parse `Content-Type` header from request headers,
   - classify whether media type is JSON.
2. Extended runtime request context with:
   - `has_content_type`,
   - `content_type_is_json`.
3. Hardened `sec4_rt_req_json(...)`:
   - after body-presence check, enforce JSON media type,
   - on mismatch, store deterministic error response:
     - status `415`,
     - content-type `application/json; charset=utf-8`,
     - payload `{"error":"content-type must be application/json"}`.
4. Added status-line mapping for `415 Unsupported Media Type`.

## Validation

Added CLI integration test in `compiler/sec4-cli/tests/json_output.rs`:

- `c_bin_http_runtime_req_json_rejects_non_json_content_type_when_clang_available`

Flow:
1. Build temp service via `--emit c-bin` with route calling `req.json(...)`.
2. Send `POST /users` with JSON-shaped body but header `Content-Type: text/plain`.
3. Assert response contains:
   - `HTTP/1.1 415 Unsupported Media Type`,
   - deterministic payload `{"error":"content-type must be application/json"}`.

## Tradeoffs and next steps

- This slice enforces media-type boundary correctness with low runtime complexity.
- Full parser-backed JSON validity and schema-aware extraction remain follow-up runtime slices.
