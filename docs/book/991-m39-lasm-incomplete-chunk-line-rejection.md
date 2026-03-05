# M39: LASM Incomplete Chunk-Line Rejection

Date: 2026-02-18
Milestone: M39-S2B (LASM async backend bootstrap)

## What changed

- LASM chunked request parsing now rejects EOF while reading a chunk-size line when the line does not terminate with `\n`.
- LASM chunked request parsing now rejects EOF while reading chunk trailers when a trailer line does not terminate with `\n`.
- Deterministic failure mapping:
  - `400 Bad Request`
  - message: `incomplete request while reading chunk size`
  - message: `incomplete request while reading chunk trailer`

## Why

Before this hardening, truncated chunk framing could be accepted in edge EOF conditions. Enforcing newline-terminated chunk metadata keeps request framing strict and deterministic.

## Validation

- `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_rejects_incomplete_chunk_size_line`
- `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_rejects_incomplete_chunk_trailer_line`
- `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_rejects_chunk_size_line_exceeding_header_limit`
- `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_400_for_incomplete_header_section`
