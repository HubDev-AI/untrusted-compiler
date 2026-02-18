# M39: LASM Chunk-Trailer Merge Into Request Headers

Date: 2026-02-18
Milestone: M39-S2B (LASM async backend bootstrap)

## What changed

- LASM chunked request reader now merges accepted trailer headers into the request header map.
- Trailer headers use the same case-insensitive merge behavior as normal request headers.
- This enables handler logic like `req.header("X-Trail")` to consume trailer-provided values.

## Why

Before this slice, LASM validated trailer syntax but discarded trailer values. That made valid trailer headers unobservable to route handlers and blocked parity for request-header reads when data is sent via trailers.

## Validation

- `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_accepts_chunked_trailer_headers`
- `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_rejects_forbidden_chunk_trailer_headers`
- `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_accepts_chunked_transfer_encoding_with_extension`
- `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_400_for_conflicting_content_length_headers`
