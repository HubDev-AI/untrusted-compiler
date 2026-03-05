# M39: LASM Chunk-Trailer Forbidden Header Validation

Date: 2026-02-18
Milestone: M39-S2B (LASM async backend bootstrap)

## What changed

- Hardened LASM chunked trailer parsing to reject forbidden trailer headers:
  - `Host`
  - `Content-Length`
  - `Transfer-Encoding`
- Added deterministic parser rejection on forbidden trailer headers:
  - `400 Bad Request`
  - message: `invalid chunk trailer: forbidden trailer header`
- Kept valid trailer behavior intact for non-forbidden header names.

## Why

Trailer headers must not override routing/framing semantics. Rejecting these names prevents ambiguous request interpretation and keeps LASM request framing deterministic.

## Validation

- `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_accepts_chunked_trailer_headers`
- `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_rejects_forbidden_chunk_trailer_headers`
- `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_accepts_chunked_transfer_encoding_with_extension`
- `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_400_for_conflicting_content_length_headers`
