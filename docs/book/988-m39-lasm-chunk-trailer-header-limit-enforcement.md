# M39: LASM Chunk-Trailer Header Limit Enforcement

Date: 2026-02-18
Milestone: M39-S2B (LASM async backend bootstrap)

## What changed

- Extended LASM `max_header_bytes` enforcement to chunk trailer sections.
- Chunk trailer bytes are now counted while reading trailers after the terminal `0` chunk.
- Oversized trailer sections now fail deterministically with:
  - `431 Request Header Fields Too Large`
  - message: `request headers exceed configured limit (<N> bytes)`
- Fixed LASM request-head header-limit diagnostics to interpolate the configured byte limit instead of emitting the literal `{max_header_bytes}` token.

## Why

Before this slice, LASM enforced header-size limits only on the request head, leaving chunk trailer sections unbounded. This created a resource-limit gap for chunked requests.

## Validation

- `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_accepts_chunked_trailer_headers`
- `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_rejects_forbidden_chunk_trailer_headers`
- `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_rejects_chunk_trailer_section_exceeding_header_limit`
- `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_431_when_header_limit_is_exceeded`
- `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_accepts_chunked_transfer_encoding_with_extension`
- `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_400_for_conflicting_content_length_headers`
