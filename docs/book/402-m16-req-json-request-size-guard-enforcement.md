# 402 M16 Slice: req.json Request-Size Guard Enforcement

This chapter documents M16-S7: runtime request-size guard enforcement for `req.json(...)`.

## What it is

A runtime hardening update in `runtime/c/sec4_runtime.c` that detects oversized HTTP request bodies and rejects them in the `req.json(...)` gate with deterministic `413 Payload Too Large`.

## Why it exists

Before this slice, oversized bodies were truncated to runtime buffer limits. That is unsafe behavior for a security-first service runtime because it can blur request boundaries and let partially captured input continue through handler logic.

## Implementation details

1. Extended runtime request state with `body_limit_exceeded`.
2. During request capture:
   - compare declared (`Content-Length`) and observed body size against runtime body capacity,
   - mark overflow when request body exceeds runtime limit.
3. In `sec4_rt_req_json(...)`:
   - reject overflow early with:
     - status `413`,
     - content-type `application/json; charset=utf-8`,
     - payload `{"error":"request body exceeds runtime limit"}`.
4. Added status mapping for:
   - `413 -> "Payload Too Large"`.

## Validation

Added CLI integration test in `compiler/sec4-cli/tests/json_output.rs`:

- `c_bin_http_runtime_req_json_rejects_oversized_body_when_clang_available`

Flow:
1. Build temp HTTP fixture (`req.json + res.ok`) via `--emit c-bin`.
2. Send `POST /users` with oversized JSON body (> runtime body limit).
3. Assert deterministic response:
   - `HTTP/1.1 413 Payload Too Large`
   - `{"error":"request body exceeds runtime limit"}`.

## Tradeoffs and next steps

- This slice provides immediate DoS-oriented boundary hardening with minimal complexity.
- Next improvements can align size limits with policy/budget settings and surface limit metadata through standard error envelopes.
