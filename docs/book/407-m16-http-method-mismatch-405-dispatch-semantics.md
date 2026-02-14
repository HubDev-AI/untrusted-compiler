# 407 M16 Slice: HTTP Method-Mismatch 405 Dispatch Semantics

This chapter documents M16-S12: runtime HTTP dispatch now distinguishes method mismatch from path-not-found and returns `405 Method Not Allowed` when appropriate.

## What it is

A runtime update in `runtime/c/sec4_runtime.c` route matching:

- if path and method both match: normal handler dispatch,
- if path matches but method differs: return `405`,
- if no path match: keep `404`.

## Why it exists

Returning `404` for method mismatches hides useful HTTP semantics and makes behavior less predictable for clients and tests. A deterministic `405` branch gives more accurate protocol behavior while preserving simple runtime architecture.

## Implementation details

1. Route scan now tracks:
   - exact match candidate,
   - first method-mismatch candidate for same path.
2. If no exact match and method-mismatch candidate exists:
   - runtime responds with:
     - status `405`,
     - body `method not allowed`.
3. Added status text mapping:
   - `405 -> "Method Not Allowed"`.

## Validation

Added integration test in `compiler/sec4-cli/tests/json_output.rs`:

- `c_bin_http_runtime_returns_405_on_method_mismatch_when_clang_available`

Flow:
1. Register `POST /users` route.
2. Send `GET /users`.
3. Assert:
   - `HTTP/1.1 405 Method Not Allowed`
   - deterministic trace header
   - body `method not allowed`.

## Tradeoffs and next steps

- This slice improves protocol correctness without introducing header-level `Allow` construction yet.
- Future enhancement can add deterministic `Allow` header emission for richer RFC-aligned responses.
