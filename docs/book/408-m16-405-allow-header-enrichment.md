# 408 M16 Slice: 405 Allow Header Enrichment

This chapter documents M16-S13: adding `Allow` header emission for runtime `405 Method Not Allowed` responses.

## What it is

A runtime update in `runtime/c/sec4_runtime.c` that enriches method-mismatch responses by emitting:

- `Allow: <methods>`

where `<methods>` is synthesized from registered routes for the matched path.

## Why it exists

M16-S12 introduced correct `405` semantics, but responses still lacked method guidance for clients. Adding `Allow` makes behavior more HTTP-complete and easier to debug, while remaining deterministic.

## Implementation details

1. Added response writer variant with optional extra headers:
   - `sec4_rt_send_response_with_extra_headers(...)`.
2. Kept existing writer entrypoint as wrapper:
   - `sec4_rt_send_response(...)`.
3. Added route-method collector for path:
   - `sec4_rt_collect_allow_methods(...)`.
4. Method-mismatch branch now:
   - builds `Allow` header from registered methods,
   - emits `405` with body `method not allowed`.

## Validation

Updated integration test:

- `c_bin_http_runtime_returns_405_on_method_mismatch_when_clang_available`

Additional assertion now verifies:

- `Allow: POST`

alongside existing `405` status, trace header, and body assertions.

## Tradeoffs and next steps

- Method listing remains deterministic and minimal for v0 runtime bootstrap.
- Future enhancement can sort methods canonically and include richer RFC-style details where needed.
