# M39: LASM Chunk-Size Line Header-Limit Enforcement

Date: 2026-02-18
Milestone: M39-S2B (LASM async backend bootstrap)

## What changed

- LASM chunked request parser now enforces `max_header_bytes` per chunk-size line.
- Oversized chunk-size lines (for example very long chunk extensions) now fail deterministically with:
  - `431 Request Header Fields Too Large`
  - message: `request headers exceed configured limit (<N> bytes)`

## Why

Trailer sections were already bounded by `max_header_bytes`, but chunk-size lines remained unbounded. This slice closes that gap and keeps chunked framing metadata bounded end-to-end.

## Validation

- `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_rejects_chunk_size_line_exceeding_header_limit`
- `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_rejects_chunk_trailer_section_exceeding_header_limit`
- `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_431_when_header_limit_is_exceeded`
- `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_accepts_chunked_trailer_headers`
