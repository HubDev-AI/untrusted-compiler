# M39: LASM Chunk-Extension Validation

Date: 2026-02-18
Milestone: M39-S2B (LASM async backend bootstrap)

## What changed

- Hardened LASM chunked request parsing in `sec4 run --backend lasm`:
  - chunk-size lines now validate every chunk extension segment after `;`.
  - extension names must satisfy HTTP token grammar.
  - extension values must be either token values or valid quoted strings.
- Added deterministic parser rejection for malformed chunk extensions:
  - `400 Bad Request`
  - message: `invalid transfer-encoding chunk extension`

## Why

LASM already accepted chunked request bodies. This slice closes a parsing gap where malformed chunk-extension syntax could pass through because only the size token was validated.

## Validation

- `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_accepts_chunked_transfer_encoding`
- `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_accepts_chunked_transfer_encoding_with_extension`
- `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_rejects_invalid_chunk_extension`
- `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_400_for_conflicting_content_length_headers`
